use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use enigo::{Button as MouseButton, Enigo, Mouse as EnigoMouse, Settings};
use futures_util::StreamExt;
use include_dir::{include_dir, Dir};
use local_ip_address::local_ip;
use mime_guess::from_path;
use mouse_position::mouse_position::Mouse as PositionMouse;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::time::sleep;
use tokio_tungstenite::accept_async;

#[derive(Serialize, Deserialize)]
struct CursorMessage {
    msg_type: String,
    delta_x: Option<f64>,
    delta_y: Option<f64>,
    button: Option<String>,
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Settings::default();
    let mut enigo = Enigo::new(&settings)?;

    let ws_stream = accept_async(stream).await?;
    let (_, mut read) = ws_stream.split();

    while let Some(Ok(msg)) = read.next().await {
        if let Ok(cursor_message) = serde_json::from_str::<CursorMessage>(&msg.to_text()?) {
            match cursor_message.msg_type.as_str() {
                "move" => {
                    if let (Some(dx), Some(dy)) = (cursor_message.delta_x, cursor_message.delta_y) {
                        match PositionMouse::get_mouse_position() {
                            PositionMouse::Position { x, y } => {
                                enigo.move_mouse(
                                    (x as f64 + dx) as i32,
                                    (y as f64 + dy) as i32,
                                    enigo::Coordinate::Abs,
                                )?;
                                sleep(Duration::from_millis(10)).await; // Introduce a small delay
                            }
                            PositionMouse::Error => {
                                eprintln!("Error getting mouse position");
                            }
                        }
                    }
                }
                "click" => {
                    if let Some(button) = cursor_message.button.as_deref() {
                        let btn = match button {
                            "left" => Some(MouseButton::Left),
                            "right" => Some(MouseButton::Right),
                            "middle" => Some(MouseButton::Middle),
                            _ => None,
                        };
                        if let Some(btn) = btn {
                            enigo.button(btn, enigo::Direction::Click)?;
                            sleep(Duration::from_millis(10)).await; // Introduce a small delay
                        }
                    }
                }
                "scroll" => {
                    if let Some(dy) = cursor_message.delta_y {
                        enigo.scroll(dy as i32, enigo::Axis::Vertical)?;
                        sleep(Duration::from_millis(10)).await; // Introduce a small delay
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
    let file_path = path.into_inner(); // Removed leading slash
    println!("Requested file path: {}", file_path);

    if let Some(file) = DIST_DIR.get_file(&file_path) {
        println!("Found file in DIST_DIR: {:?}", file.path());
        let mime_type = from_path(file.path()).first_or_octet_stream();
        HttpResponse::Ok()
            .content_type(mime_type.as_ref())
            .body(file.contents().to_vec())
    } else {
        // Check if the path has an extension
        if Path::new(&file_path).extension().is_none() {
            // No extension, serve index.html for SPA routing
            let index_file = DIST_DIR.get_file("index.html").unwrap();
            let mime_type = from_path(index_file.path()).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(index_file.contents())
        } else {
            // Static asset not found, return 404 Not Found
            println!("Static asset not found: {}", file_path);
            HttpResponse::NotFound().finish()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await?;

    // Get the local IP address
    let local_ip = local_ip().unwrap();
    println!("Go to website: http://{}:8081", local_ip);

    let web_server =
        HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(serve_file)))
            .bind("0.0.0.0:8081")?
            .run();

    spawn(async move {
        while let Ok((stream, _)) = tcp_listener.accept().await {
            spawn(handle_connection(stream));
        }
    });

    web_server.await?;
    Ok(())
}
