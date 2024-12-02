use crate::mouse_controller::MouseController;
use enigo::Button;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use tungstenite::{accept, Message};

pub fn start_websocket_server(controller: Arc<MouseController>) {
    thread::spawn(move || {
        let server = TcpListener::bind("0.0.0.0:9001").expect("Failed to bind to address");
        for stream in server.incoming().flatten() {
            if let Err(e) = stream.set_nodelay(true) {
                eprintln!("Failed to set TCP_NODELAY: {}", e);
            }

            let ws_controller = Arc::clone(&controller);
            thread::spawn(move || {
                let mut websocket = match accept(stream) {
                    Ok(ws) => ws,
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                        return;
                    }
                };
                println!("New WebSocket connection established.");

                loop {
                    match websocket.read() {
                        Ok(Message::Binary(data)) if !data.is_empty() => match data[0] {
                            0x01 if data.len() == 9 => {
                                let dx = i32::from_le_bytes(data[1..5].try_into().unwrap());
                                let dy = i32::from_le_bytes(data[5..9].try_into().unwrap());
                                if let Err(e) = ws_controller.mouse_move_batch(vec![(dx, dy)]) {
                                    eprintln!("Error moving mouse: {}", e);
                                }
                            }
                            0x03 if data.len() == 2 => {
                                let button = match data[1] {
                                    1 => Some(Button::Left),
                                    3 => Some(Button::Right),
                                    _ => None,
                                };
                                if let Some(btn) = button {
                                    if let Err(e) = ws_controller.click(btn) {
                                        eprintln!("Error clicking mouse: {}", e);
                                    }
                                }
                            }
                            _ => eprintln!("Unknown message type"),
                        },
                        Ok(Message::Close(_)) => break,
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    });
}
