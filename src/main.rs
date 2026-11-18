use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::{Method, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use clap::Parser;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::Infallible,
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast;
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use zpl_rs::{Dpi, RenderOptions, render_bytes_with_options};

/// The enum and the dots-per-inch factor must agree.
const DPI: Dpi = Dpi::Dpi203;
const DOTS_PER_INCH: f32 = 203.0;

/// libzpl (Go FFI) is not thread-safe for concurrent renders.
static RENDER_LOCK: Mutex<()> = Mutex::new(());

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Default label width in inches
    #[arg(short = 'x', long, default_value_t = 2.25)]
    width: f32,

    /// Default label height in inches
    #[arg(short = 'y', long, default_value_t = 1.25)]
    height: f32,

    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Interface to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    interface: String,

    /// Directory containing the built Nuxt app (optional)
    #[arg(long)]
    web_dir: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct LabelConfig {
    width: f32,
    height: f32,
}

struct Printer {
    /// Base64-encoded PNGs, encoded once and shared by every subscriber.
    sender: broadcast::Sender<Arc<str>>,
    config: LabelConfig,
}

#[derive(Clone)]
struct AppState {
    default_config: LabelConfig,
    printers: Arc<Mutex<HashMap<String, Printer>>>,
}

impl AppState {
    /// Runs `f` on the named printer, creating it with the default config if needed.
    fn with_printer<R>(&self, name: String, f: impl FnOnce(&mut Printer) -> R) -> R {
        let mut printers = self.printers.lock().unwrap();
        let printer = printers.entry(name).or_insert_with(|| Printer {
            sender: broadcast::channel(32).0,
            config: self.default_config,
        });
        f(printer)
    }
}

fn render(zpl: &[u8], config: LabelConfig) -> zpl_rs::Result<Vec<u8>> {
    let options = RenderOptions::new().dpi(DPI).size(
        (config.width * DOTS_PER_INCH) as i32,
        (config.height * DOTS_PER_INCH) as i32,
    );
    let _guard = RENDER_LOCK.lock().unwrap();
    render_bytes_with_options(zpl, &options)
}

async fn get_config(Path(name): Path<String>, State(state): State<AppState>) -> Json<LabelConfig> {
    let printers = state.printers.lock().unwrap();
    Json(printers.get(&name).map_or(state.default_config, |p| p.config))
}

async fn post_config(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(config): Json<LabelConfig>,
) {
    state.with_printer(name, |p| p.config = config);
}

async fn get_stream(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.with_printer(name, |p| p.sender.subscribe());
    let stream = BroadcastStream::new(receiver)
        .filter_map(|msg| msg.ok().map(|png| Ok(Event::default().data(&*png))));
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30)))
}

async fn post_print(
    Path(name): Path<String>,
    State(state): State<AppState>,
    body: Bytes,
) -> StatusCode {
    let (config, sender) = state.with_printer(name.clone(), |p| (p.config, p.sender.clone()));

    match tokio::task::spawn_blocking(move || render(&body, config)).await {
        Ok(Ok(png)) => {
            let _ = sender.send(BASE64.encode(png).into());
            info!("Rendered label for printer '{name}'");
            StatusCode::OK
        }
        Ok(Err(e)) => {
            error!("Failed to render ZPL for printer '{name}': {e}");
            StatusCode::BAD_REQUEST
        }
        Err(e) => {
            error!("Render task failed for printer '{name}': {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn app(default_config: LabelConfig, web_dir: Option<PathBuf>) -> Router {
    let state = AppState { default_config, printers: Arc::default() };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    let router = Router::new()
        .route("/{printer_name}/pstprnt", post(post_print))
        .route("/{printer_name}/stream", get(get_stream))
        .route("/{printer_name}/config", get(get_config).post(post_config))
        .with_state(state);

    match web_dir {
        Some(dir) => router.fallback_service(
            ServeDir::new(&dir).fallback(ServeFile::new(dir.join("index.html"))),
        ),
        None => router,
    }
    .layer(cors)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    colog::init();
    let args = Args::parse();

    let addr = format!("{}:{}", args.interface, args.port);
    info!("Zebra Emulator listening on http://{addr}");
    info!("Default label size: {}x{} inches", args.width, args.height);

    let app = app(LabelConfig { width: args.width, height: args.height }, args.web_dir);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, response::Response};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    const DEFAULT: LabelConfig = LabelConfig { width: 2.25, height: 1.25 };
    const HELLO: &str = "^XA^FO50,50^ADN,36,20^FDHello^FS^XZ";

    async fn send(app: &Router, method: Method, uri: &str, body: &str) -> Response {
        let mut req = Request::builder().method(method).uri(uri);
        if body.starts_with('{') {
            req = req.header("content-type", "application/json");
        }
        app.clone().oneshot(req.body(Body::from(body.to_owned())).unwrap()).await.unwrap()
    }

    async fn body_text(res: Response) -> String {
        String::from_utf8(res.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap()
    }

    /// Reads SSE frames until one carries a label, and returns its PNG (width, height).
    async fn next_label(body: &mut Body) -> (u32, u32) {
        loop {
            let frame = tokio::time::timeout(Duration::from_secs(10), body.frame())
                .await
                .expect("no label received within 10s")
                .unwrap()
                .unwrap();
            let text = String::from_utf8(frame.into_data().unwrap().to_vec()).unwrap();
            if let Some(data) = text.lines().find_map(|l| l.strip_prefix("data:")) {
                let png = BASE64.decode(data.trim()).unwrap();
                assert_eq!(&png[1..4], b"PNG");
                let dim = |i: usize| u32::from_be_bytes(png[i..i + 4].try_into().unwrap());
                return (dim(16), dim(20));
            }
        }
    }

    #[tokio::test]
    async fn config_defaults_then_persists_per_printer() {
        let app = app(DEFAULT, None);

        let res = send(&app, Method::GET, "/a/config", "").await;
        assert_eq!(body_text(res).await, r#"{"width":2.25,"height":1.25}"#);

        let res = send(&app, Method::POST, "/a/config", r#"{"width":4,"height":6}"#).await;
        assert_eq!(res.status(), StatusCode::OK);

        let res = send(&app, Method::GET, "/a/config", "").await;
        assert_eq!(body_text(res).await, r#"{"width":4.0,"height":6.0}"#);
        let res = send(&app, Method::GET, "/b/config", "").await;
        assert_eq!(body_text(res).await, r#"{"width":2.25,"height":1.25}"#);
    }

    #[tokio::test]
    async fn invalid_config_is_rejected() {
        let app = app(DEFAULT, None);
        let res = send(&app, Method::POST, "/a/config", r#"{"width":"big"}"#).await;
        assert!(res.status().is_client_error());
    }

    #[tokio::test]
    async fn print_streams_png_sized_from_printer_config() {
        let app = app(DEFAULT, None);
        let mut stream = send(&app, Method::GET, "/p/stream", "").await;
        assert_eq!(stream.headers()["content-type"], "text/event-stream");

        let res = send(&app, Method::POST, "/p/pstprnt", HELLO).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(next_label(stream.body_mut()).await, (456, 253)); // 2.25x1.25in @ 203dpi

        send(&app, Method::POST, "/p/config", r#"{"width":4,"height":6}"#).await;
        send(&app, Method::POST, "/p/pstprnt", HELLO).await;
        assert_eq!(next_label(stream.body_mut()).await, (812, 1218));
    }

    #[tokio::test]
    async fn labels_only_reach_their_own_printer() {
        let app = app(DEFAULT, None);
        let mut a = send(&app, Method::GET, "/a/stream", "").await;
        let mut b = send(&app, Method::GET, "/b/stream", "").await;

        send(&app, Method::POST, "/b/pstprnt", HELLO).await;
        next_label(b.body_mut()).await;
        let nothing = tokio::time::timeout(Duration::from_millis(300), a.body_mut().frame()).await;
        assert!(nothing.is_err(), "printer 'a' received a label sent to 'b'");
    }

    #[tokio::test]
    async fn concurrent_prints_all_render() {
        let app = app(DEFAULT, None);
        let mut stream = send(&app, Method::GET, "/c/stream", "").await;

        let prints: Vec<_> = (0..8)
            .map(|_| {
                let app = app.clone();
                tokio::spawn(async move { send(&app, Method::POST, "/c/pstprnt", HELLO).await })
            })
            .collect();
        for print in prints {
            assert_eq!(print.await.unwrap().status(), StatusCode::OK);
        }
        for _ in 0..8 {
            assert_eq!(next_label(stream.body_mut()).await, (456, 253));
        }
    }

    #[tokio::test]
    async fn web_dir_serves_spa_with_index_fallback() {
        let dir = std::env::temp_dir().join(format!("zebra-web-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), "<p>spa</p>").unwrap();
        let app = app(DEFAULT, Some(dir.clone()));

        for uri in ["/", "/some-printer-1234"] {
            let res = send(&app, Method::GET, uri, "").await;
            assert_eq!(res.status(), StatusCode::OK, "{uri}");
            assert_eq!(body_text(res).await, "<p>spa</p>", "{uri}");
        }
        // API routes still win over the SPA fallback.
        let res = send(&app, Method::GET, "/x/config", "").await;
        assert!(body_text(res).await.starts_with('{'));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn cors_allows_cross_origin_prints() {
        let app = app(DEFAULT, None);
        let req = Request::builder()
            .method(Method::OPTIONS)
            .uri("/p/pstprnt")
            .header("origin", "http://example.com")
            .header("access-control-request-method", "POST")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.headers()["access-control-allow-origin"], "*");
    }
}
