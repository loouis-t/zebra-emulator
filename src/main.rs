use axum::{
    Router, body::Bytes, extract::State, http::StatusCode, response::IntoResponse, routing::post,
};

use clap::Parser;
use log::{error, info};
use open;
use std::{error::Error, fs::File, io::Write, time::Duration};
use tower_http::cors::{Any, CorsLayer};
use zpl_rs::{Dpi, RenderOptions, render_with_options};

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
    let zpl = String::from_utf8_lossy(&body);
    let options = RenderOptions::new()
        .dpi(Dpi::Dpi203)
        .size((args.width * 203.0) as i32, (args.height * 203.0) as i32);

    let png_bytes = match render_with_options(&zpl, &options) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to render ZPL: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    match display_png(png_bytes.as_slice()) {
        Ok(_) => (),
        Err(_) => {
            error!("Could not open PNG in default viewer");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    info!("Successfully rendered and displayed label");
    StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    colog::init();
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

    let listening_addr = format!("{}:{}", args.interface, args.port);
    info!("Zebra Emulator is listening on {}", listening_addr);
    info!("ZPL format is set to {}x{}", args.width, args.height);

    let listener =
        tokio::net::TcpListener::bind(listening_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
