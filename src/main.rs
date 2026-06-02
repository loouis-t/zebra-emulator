use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, sse::{Event, KeepAlive, Sse}},
    routing::{get, post},
};
use tower_http::services::{ServeDir, ServeFile};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use clap::Parser;
use futures::StreamExt;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible, error::Error, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::{Any, CorsLayer};
use zpl_rs::{Dpi, RenderOptions, render_with_options};

#[derive(Parser, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
struct LabelConfig {
    width: f32,
    height: f32,
}

struct PrinterState {
    sender: broadcast::Sender<Bytes>,
    last_image: Option<Bytes>,
    config: LabelConfig,
}

type PrinterMap = Arc<Mutex<HashMap<String, PrinterState>>>;

#[derive(Clone)]
struct AppState {
    args: Args,
    printers: PrinterMap,
    render_lock: Arc<Mutex<()>>,
}

impl AppState {
    fn new_printer(&self) -> PrinterState {
        PrinterState {
            sender: broadcast::channel::<Bytes>(32).0,
            last_image: None,
            config: LabelConfig { width: self.args.width, height: self.args.height },
        }
    }
}

async fn get_config(
    Path(printer_name): Path<String>,
    State(state): State<AppState>,
) -> Json<LabelConfig> {
    let map = state.printers.lock().await;
    let config = map.get(&printer_name)
        .map(|p| p.config.clone())
        .unwrap_or(LabelConfig { width: state.args.width, height: state.args.height });
    Json(config)
}

async fn post_config(
    Path(printer_name): Path<String>,
    State(state): State<AppState>,
    Json(config): Json<LabelConfig>,
) -> StatusCode {
    let mut map = state.printers.lock().await;
    let entry = map.entry(printer_name).or_insert_with(|| state.new_printer());
    entry.config = config;
    StatusCode::OK
}

async fn get_stream(
    Path(printer_name): Path<String>,
    State(state): State<AppState>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let receiver = {
        let mut map = state.printers.lock().await;
        let entry = map.entry(printer_name).or_insert_with(|| state.new_printer());
        entry.sender.subscribe()
    };

    let stream = BroadcastStream::new(receiver).filter_map(|result| async move {
        result.ok().map(|bytes| Ok(Event::default().data(BASE64.encode(&bytes))))
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30)))
}

async fn post_print(
    Path(printer_name): Path<String>,
    State(state): State<AppState>,
    body: Bytes,
) -> impl IntoResponse {
    let zpl = String::from_utf8_lossy(&body);

    let config = {
        let map = state.printers.lock().await;
        map.get(&printer_name)
            .map(|p| p.config.clone())
            .unwrap_or(LabelConfig { width: state.args.width, height: state.args.height })
    };

    let options = RenderOptions::new()
        .dpi(Dpi::Dpi203)
        .size((config.width * 203.0) as i32, (config.height * 203.0) as i32);

    // libzpl.so (Go FFI) is not thread-safe for concurrent renders
    let png_bytes = {
        let _guard = state.render_lock.lock().await;
        match render_with_options(&zpl, &options) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to render ZPL: {}", e);
                return StatusCode::BAD_REQUEST;
            }
        }
    };

    let png = Bytes::from(png_bytes);

    let mut map = state.printers.lock().await;
    let entry = map.entry(printer_name.clone()).or_insert_with(|| state.new_printer());
    entry.last_image = Some(png.clone());
    let _ = entry.sender.send(png);

    info!("Rendered label for printer '{}'", printer_name);
    StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    colog::init();
    let args = Args::parse();

    let state = AppState {
        args: args.clone(),
        printers: Arc::new(Mutex::new(HashMap::new())),
        render_lock: Arc::new(Mutex::new(())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec!["GET".parse()?, "POST".parse()?])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    let router = Router::new()
        .route("/{printer_name}/pstprnt", post(post_print))
        .route("/{printer_name}/stream", get(get_stream))
        .route("/{printer_name}/config", get(get_config).post(post_config))
        .with_state(state);

    let app = match &args.web_dir {
        Some(dir) => router.fallback_service(
            ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html"))),
        ),
        None => router,
    }
    .layer(cors);

    let listening_addr = format!("{}:{}", args.interface, args.port);
    info!("Zebra Emulator listening on http://{}", listening_addr);
    info!("Default label size: {}x{} inches", args.width, args.height);

    let listener = tokio::net::TcpListener::bind(listening_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
