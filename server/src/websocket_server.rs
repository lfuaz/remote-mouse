use crate::mouse_controller::MouseController;
use enigo::Button;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::{accept, Error as WsError, Message};

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

                let mut move_batch = Vec::new();
                let mut last_process_time = Instant::now();
                const BATCH_TIMEOUT: Duration = Duration::from_millis(16); // ~60fps

                loop {
                    match websocket.read() {
                        Ok(msg) => {
                            if let Message::Binary(data) = msg {
                                if !data.is_empty() {
                                    match data[0] {
                                        0x01 if data.len() == 9 => {
                                            let dx =
                                                i32::from_le_bytes(data[1..5].try_into().unwrap());
                                            let dy =
                                                i32::from_le_bytes(data[5..9].try_into().unwrap());
                                            move_batch.push((dx, dy));

                                            // Process batch if we have enough moves or timeout reached
                                            if move_batch.len() >= 5
                                                || last_process_time.elapsed() >= BATCH_TIMEOUT
                                            {
                                                if let Err(e) = ws_controller
                                                    .mouse_move_batch(move_batch.clone())
                                                {
                                                    eprintln!("Error moving mouse: {}", e);
                                                }
                                                move_batch.clear();
                                                last_process_time = Instant::now();
                                            }
                                        }
                                        0x03 if data.len() == 2 => {
                                            // Process any pending moves before handling click
                                            if !move_batch.is_empty() {
                                                if let Err(e) = ws_controller
                                                    .mouse_move_batch(move_batch.clone())
                                                {
                                                    eprintln!("Error moving mouse: {}", e);
                                                }
                                                move_batch.clear();
                                                last_process_time = Instant::now();
                                            }

                                            let button = match data[1] {
                                                1 => Some(Button::Left),
                                                2 => Some(Button::Right),
                                                _ => None,
                                            };

                                            if let Some(btn) = button {
                                                if let Err(e) = ws_controller.click(btn) {
                                                    eprintln!("Error clicking mouse: {}", e);
                                                }
                                            }
                                        }
                                        _ => eprintln!("Unknown message type"),
                                    }
                                }
                            } else if let Message::Close(_) = msg {
                                break;
                            }
                        }
                        Err(WsError::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // Just continue if no data is available
                            continue;
                        }
                        Err(e) => {
                            eprintln!("WebSocket error: {}", e);
                            break;
                        }
                    }

                    // Process any pending moves in batch if timeout reached
                    if !move_batch.is_empty() && last_process_time.elapsed() >= BATCH_TIMEOUT {
                        if let Err(e) = ws_controller.mouse_move_batch(move_batch.clone()) {
                            eprintln!("Error moving mouse: {}", e);
                        }
                        move_batch.clear();
                        last_process_time = Instant::now();
                    }

                    thread::sleep(Duration::from_millis(1));
                }
            });
        }
    });
}
