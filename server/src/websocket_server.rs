use crate::mouse_controller::MouseController;
use enigo::Button;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::http::StatusCode;
use tungstenite::{accept_hdr, Message};

static CONNECTED: AtomicBool = AtomicBool::new(false);

fn reject(status: StatusCode) -> ErrorResponse {
    let mut res = ErrorResponse::new(None);
    *res.status_mut() = status;
    res
}

pub fn start_websocket_server(
    listener: TcpListener,
    controller: Arc<MouseController>,
    key: String,
) {
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = stream.set_nodelay(true);
            let controller = Arc::clone(&controller);
            let expected = format!("key={}", key);
            thread::spawn(move || {
                let check = |req: &Request, res: Response| {
                    if req.uri().query() != Some(expected.as_str()) {
                        return Err(reject(StatusCode::UNAUTHORIZED));
                    }
                    // Only one client may hold the mouse at a time.
                    if CONNECTED.swap(true, Ordering::SeqCst) {
                        return Err(reject(StatusCode::CONFLICT));
                    }
                    Ok(res)
                };
                let mut websocket = match accept_hdr(stream, check) {
                    Ok(ws) => ws,
                    Err(e) => return eprintln!("Rejected connection: {}", e),
                };
                println!("New WebSocket connection established.");

                // Client already coalesces moves per animation frame, so apply each message immediately.
                while let Ok(msg) = websocket.read() {
                    let data = match msg {
                        Message::Binary(data) => data,
                        Message::Close(_) => break,
                        _ => continue,
                    };
                    let result = match data[..] {
                        [0x01, ..] if data.len() == 9 => controller.mouse_move(
                            i32::from_le_bytes(data[1..5].try_into().unwrap()),
                            i32::from_le_bytes(data[5..9].try_into().unwrap()),
                        ),
                        [0x03, 1] => controller.click(Button::Left),
                        [0x03, 2] => controller.click(Button::Right),
                        _ => Err("Unknown message type".into()),
                    };
                    if let Err(e) = result {
                        eprintln!("{}", e);
                    }
                }
                CONNECTED.store(false, Ordering::SeqCst);
                println!("WebSocket connection closed.");
            });
        }
    });
}
