use axum::{
    routing::post,
    Router,
    body::Bytes,
    response::IntoResponse,
    http::StatusCode,
};

use zpl_forge::{ZplEngine, Unit, Resolution};
use zpl_forge::forge::png::PngBackend;
use std::{
    collections::HashMap,
    time::Duration,
};
use minifb::{Window, WindowOptions};
use tower_http::cors::{CorsLayer, Any};

async fn handle_post(body: Bytes) -> impl IntoResponse {
    let zpl_input = String::from_utf8_lossy(&body);

    // Create ZPL engine
    let engine = match ZplEngine::new(
        &zpl_input,
        Unit::Inches(2.25),
        Unit::Inches(1.25),
        Resolution::Dpi203,
    ) {
        Ok(e) => e,
        Err(_e) => return StatusCode::BAD_REQUEST,
    };

    // Render to PNG bytes
    let backend = PngBackend::new();
    let png_bytes = match engine.render(backend, &HashMap::new()) {
        Ok(bytes) => bytes,
        Err(_e) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    // Decode PNG
    let img = match image::load_from_memory(&png_bytes) {
        Ok(img) => img.to_rgba8(),
        Err(_e) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let (width, height) = img.dimensions();
    let buffer: Vec<u32> = img
        .pixels()
        .map(|p| {
            let r = p[0] as u32;
            let g = p[1] as u32;
            let b = p[2] as u32;
            let a = p[3] as u32;
            (a << 24) | (r << 16) | (g << 8) | b
        })
        .collect();

    tokio::spawn(async move {
        let mut window = match Window::new(
            "ZPL Preview",
            width as usize,
            height as usize,
            WindowOptions::default(),
        ) {
            Ok(win) => win,
            Err(_) => return,
        };

        while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
            window.update_with_buffer(&buffer, width as usize, height as usize).unwrap();
        }
    });

    StatusCode::OK
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec!["POST".parse().unwrap()])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    let app = Router::new()
        .route("/pstprnt", post(handle_post))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
