use crate::mouse_controller::MouseController;
use enigo::Button;
use std::collections::VecDeque;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::{accept, Message};

pub fn start_websocket_server(controller: Arc<MouseController>) {
    thread::spawn(move || {
        let server = TcpListener::bind("0.0.0.0:9001").expect("Failed to bind to address");
        for stream in server.incoming().flatten() {
            // Set TCP_NODELAY to reduce latency
            if let Ok(tcp_stream) = stream.try_clone() {
                tcp_stream.set_nodelay(true).unwrap_or_default();
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

                let mut move_buffer = VecDeque::with_capacity(32);
                let mut last_flush = Instant::now();
                let batch_timeout = Duration::from_millis(16); // ~60Hz update rate

                loop {
                    match websocket.read() {
                        Ok(Message::Binary(data)) if !data.is_empty() => {
                            match data[0] {
                                0x01 if data.len() == 9 => {
                                    let dx = i32::from_le_bytes(data[1..5].try_into().unwrap());
                                    let dy = i32::from_le_bytes(data[5..9].try_into().unwrap());

                                    move_buffer.push_back((dx, dy));

                                    // Flush buffer if it's full or enough time has passed
                                    if move_buffer.len() >= 32
                                        || last_flush.elapsed() >= batch_timeout
                                    {
                                        if let Err(e) = ws_controller
                                            .mouse_move_batch(move_buffer.drain(..).collect())
                                        {
                                            eprintln!("Error moving mouse: {}", e);
                                        }
                                        last_flush = Instant::now();
                                    }
                                }
                                0x03 if data.len() == 2 => {
                                    // Flush any pending movements before handling clicks
                                    if !move_buffer.is_empty() {
                                        if let Err(e) = ws_controller
                                            .mouse_move_batch(move_buffer.drain(..).collect())
                                        {
                                            eprintln!("Error moving mouse: {}", e);
                                        }
                                    }

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
                                _ => eprintln!("Unknown or invalid message type: {}", data[0]),
                            }
                        }
                        Ok(Message::Close(_)) => {
                            println!("WebSocket connection closed.");
                            break;
                        }
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
