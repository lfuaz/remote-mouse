use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use enigo::{
    Button, Coordinate,
    Direction::{Press, Release},
    Enigo, Mouse, Settings,
};
use futures_util::StreamExt;
use include_dir::{include_dir, Dir};
use local_ip_address::local_ip;
use log::{error, info};
use mime_guess::from_path;
use mouse_position::mouse_position::Mouse as PositionMouse;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;

#[derive(Serialize, Deserialize)]
struct CursorMessage {
    msg_type: String,
    delta_x: Option<f64>,
    delta_y: Option<f64>,
    button: Option<String>,
}

async fn handle_connection(stream: tokio::net::TcpStream, enigo: Arc<Mutex<Enigo>>) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (_, mut read) = ws_stream.split();

    while let Some(Ok(msg)) = read.next().await {
        if let Ok(cursor_message) = serde_json::from_str::<CursorMessage>(&msg.to_text()?) {
            match cursor_message.msg_type.as_str() {
                "move" => {
                    if let (Some(dx), Some(dy)) = (cursor_message.delta_x, cursor_message.delta_y) {
                        match PositionMouse::get_mouse_position() {
                            PositionMouse::Position { x, y } => {
                                let mut enigo = enigo.lock().await;
                                if let Err(e) = enigo.move_mouse(
                                    (x as f64 + dx) as i32,
                                    (y as f64 + dy) as i32,
                                    Coordinate::Abs,
                                ) {
                                    error!("Failed to move mouse: {}", e);
                                }
                            }
                            PositionMouse::Error => {
                                error!("Error getting mouse position");
                            }
                        }
                    }
                }
                "click" => {
                    if let Some(button) = cursor_message.button.as_deref() {
                        let btn = match button {
                            "left" => Some(Button::Left),
                            "right" => Some(Button::Right),
                            "middle" => Some(Button::Middle),
                            _ => None,
                        };
                        if let Some(btn) = btn {
                            let mut enigo = enigo.lock().await;
                            if let Err(e) = enigo.button(btn, Press) {
                                error!("Failed to press button: {}", e);
                            }
                            if let Err(e) = enigo.button(btn, Release) {
                                error!("Failed to release button: {}", e);
                            }
                        }
                    }
                }
                "scroll" => {
                    if let Some(dy) = cursor_message.delta_y {
                        let mut enigo = enigo.lock().await;
                        if let Err(e) = enigo.scroll(dy as i32, enigo::Axis::Vertical) {
                            error!("Failed to scroll: {}", e);
                        }
                    }
                }
                _ => (),
            }
        }
    }
    Ok(())
}

static DIST_DIR: Dir = include_dir!("./dist");

async fn serve_file(path: web::Path<String>, _req: HttpRequest) -> impl Responder {
    let file_path = path.into_inner();
    println!("Requested file path: {}", file_path);

    if let Some(file) = DIST_DIR.get_file(&file_path) {
        println!("Found file in DIST_DIR: {:?}", file.path());
        let mime_type = from_path(file.path()).first_or_octet_stream();
        HttpResponse::Ok()
            .content_type(mime_type.as_ref())
            .body(file.contents().to_vec())
    } else {
        if Path::new(&file_path).extension().is_none() {
            if let Some(index_file) = DIST_DIR.get_file("index.html") {
                let mime_type = from_path(index_file.path()).first_or_octet_stream();
                HttpResponse::Ok()
                    .content_type(mime_type.as_ref())
                    .body(index_file.contents())
            } else {
                println!("index.html not found in DIST_DIR");
                HttpResponse::NotFound().finish()
            }
        } else {
            println!("Static asset not found: {}", file_path);
            HttpResponse::NotFound().finish()
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger with the info level
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await?;

    let local_ip = local_ip()?;
    let url = format!("http://{}:8081", local_ip);
    info!("Go to website: {}", url);

    let code = QrCode::new(&url)?;

    // Render QR code to console
    let qr_code_console = code.render::<qrcode::render::unicode::Dense1x2>().build();
    println!("{}", qr_code_console);

    let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));

    let web_server =
        HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(serve_file)))
            .bind("0.0.0.0:8081")?
            .run();

    spawn(async move {
        loop {
            match tcp_listener.accept().await {
                Ok((stream, _)) => {
                    let enigo = enigo.clone();
                    spawn(handle_connection(stream, enigo));
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    tokio::select! {
        res = web_server => {
            if let Err(e) = res {
                error!("Web server error: {}", e);
            }
        },
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        },
    }

    Ok(())
}
