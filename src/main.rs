use axum::{
    Router, body::Bytes, extract::State, http::StatusCode, response::IntoResponse, routing::post,
};

use clap::Parser;
use open;
use std::{collections::HashMap, error::Error, fs::File, io::Write, time::Duration};
use tower_http::cors::{Any, CorsLayer};
use zpl_forge::{Resolution, Unit, ZplEngine, forge::png::PngBackend};

/// Simple program to serve a POST endpoint that accepts ZPL
/// and renders it to PNG, then opens the PNG in the default viewer.
/// The purpose is to simulate the HTTP server that Zebra printers host,
/// allowing to print through local network.
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Label width in inches
    #[arg(short = 'x', long, default_value_t = 2.25)]
    width: f32,

    /// Label height in inches
    #[arg(short = 'y', long, default_value_t = 1.25)]
    height: f32,

    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Interface to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    interface: String,
}

fn display_png(png_data: &[u8]) -> std::io::Result<()> {
    let path = std::env::temp_dir().join("zebra_label.png");
    File::create(&path)?.write_all(png_data)?;
    open::that(&path)?;
    Ok(())
}

async fn post_print(State(args): State<Args>, body: Bytes) -> impl IntoResponse {
    let engine = match ZplEngine::new(
        &String::from_utf8_lossy(&body),
        Unit::Inches(args.width),
        Unit::Inches(args.height),
        Resolution::Dpi203,
    ) {
        Ok(e) => e,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let png_bytes = match engine.render(PngBackend::new(), &HashMap::new()) {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    match display_png(png_bytes.as_slice()) {
        Ok(_) => (),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    }

    StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec!["POST".parse()?])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    let app = Router::new()
        .route("/pstprnt", post(post_print))
        .with_state(args.clone())
        .layer(cors);

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", args.interface, args.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
