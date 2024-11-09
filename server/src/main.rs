use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use include_dir::{include_dir, Dir};
use local_ip_address::local_ip;
use mime_guess::from_path;
use qrcode::QrCode;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use tungstenite::accept;
use tungstenite::protocol::Message;
use winapi::shared::windef::POINT;
use winapi::um::winuser::{
    mouse_event, GetCursorPos, SetCursorPos, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
};

/// Struct to control the mouse.
struct MouseController;

impl MouseController {
    pub fn mouse_move_relative(&self, dx: i32, dy: i32) -> Result<(), String> {
        unsafe {
            let mut point: POINT = std::mem::zeroed();

            // Retrieve the current cursor position
            if GetCursorPos(&mut point) == 0 {
                return Err("Failed to get cursor position".into());
            }

            // Calculate the new position
            let new_x = point.x + dx;
            let new_y = point.y + dy;

            // Set the new cursor position
            if SetCursorPos(new_x, new_y) == 0 {
                return Err("Failed to set cursor position".into());
            }

            Ok(())
        }
    }

    pub fn click(&self, button_code: u8) -> Result<(), String> {
        unsafe {
            match button_code {
                0x01 => {
                    // Left click
                    mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                }
                0x02 => {
                    // Right click
                    mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
                }
                0x03 => {
                    // Middle click
                    mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0, 0);
                }
                _ => return Err("Invalid button code".into()),
            }
            Ok(())
        }
    }
}

static DIST_DIR: Dir = include_dir!("./dist");

async fn serve_file(path: web::Path<String>, _req: HttpRequest) -> impl Responder {
    let file_path = path.into_inner();

    if let Some(file) = DIST_DIR.get_file(&file_path) {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the mouse controller
    let controller = Arc::new(MouseController);

    // Start the WebSocket server in a new thread
    let ws_controller = Arc::clone(&controller);
    thread::spawn(move || {
        // Bind the WebSocket server to a local address
        let server = TcpListener::bind("0.0.0.0:9001").expect("Failed to bind to address");
        // Accept incoming connections in a loop
        for stream in server.incoming() {
            match stream {
                Ok(stream) => {
                    let ws_controller = Arc::clone(&ws_controller);
                    thread::spawn(move || {
                        let mut websocket = accept(stream).expect("Failed to accept connection");
                        println!("New WebSocket connection established.");

                        loop {
                            match websocket.read() {
                                Ok(Message::Binary(data)) => {
                                    if data.is_empty() {
                                        continue;
                                    }

                                    match data[0] {
                                        0x01 => {
                                            // Mouse move
                                            if data.len() != 9 {
                                                eprintln!("Invalid move data length");
                                                continue;
                                            }
                                            let dx = i32::from_le_bytes([
                                                data[1], data[2], data[3], data[4],
                                            ]);
                                            let dy = i32::from_le_bytes([
                                                data[5], data[6], data[7], data[8],
                                            ]);
                                            if let Err(e) =
                                                ws_controller.mouse_move_relative(dx, dy)
                                            {
                                                eprintln!("Error moving mouse: {}", e);
                                            }
                                        }
                                        0x03 => {
                                            // Mouse click
                                            if data.len() != 2 {
                                                eprintln!("Invalid click data length");
                                                continue;
                                            }
                                            if let Err(e) = ws_controller.click(data[1]) {
                                                eprintln!("Error clicking mouse: {}", e);
                                            }
                                        }
                                        _ => eprintln!("Unknown message type: {}", data[0]),
                                    }
                                }
                                Ok(Message::Close(_)) => {
                                    println!("WebSocket connection closed.");
                                    break;
                                }
                                Ok(_) => {} // Ignore other message types
                                Err(e) => {
                                    eprintln!("WebSocket error: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    let local_ip = local_ip().unwrap();
    let url = format!("http://{}:8081", local_ip);
    println!("Go to website: {}", url);

    let code = QrCode::new(&url).unwrap();

    // Render QR code to console
    let qr_code_console = code.render::<qrcode::render::unicode::Dense1x2>().build();
    println!("{}", qr_code_console);

    // Start the HTTP server to serve static files
    HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(serve_file)))
        .bind("0.0.0.0:8081")?
        .run()
        .await
}
