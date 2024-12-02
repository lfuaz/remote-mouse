use crate::mouse_controller::MouseController;
use enigo::Button;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use tungstenite::accept;
use tungstenite::protocol::Message;

pub fn start_websocket_server(controller: Arc<MouseController>) {
    thread::spawn(move || {
        let server = TcpListener::bind("0.0.0.0:9001").expect("Failed to bind to address");
        for stream in server.incoming() {
            match stream {
                Ok(stream) => {
                    let ws_controller = Arc::clone(&controller);
                    thread::spawn(move || {
                        let mut websocket = accept(stream).expect("Failed to accept connection");
                        println!("New WebSocket connection established.");

                        let mut move_batch = Vec::new();
                        let batch_size = 5;

                        loop {
                            match websocket.read() {
                                Ok(Message::Binary(data)) => {
                                    if data.is_empty() {
                                        continue;
                                    }

                                    match data[0] {
                                        0x01 => {
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

                                            move_batch.push((dx, dy));
                                            if move_batch.len() >= batch_size {
                                                if let Err(e) = ws_controller
                                                    .mouse_move_batch(move_batch.clone())
                                                {
                                                    eprintln!("Error moving mouse batch: {}", e);
                                                }
                                                move_batch.clear();
                                            }
                                        }
                                        0x03 => {
                                            if data.len() != 2 {
                                                eprintln!("Invalid click data length");
                                                continue;
                                            }
                                            // Convert u8 to Button enum
                                            let button = match data[1] {
                                                1 => Button::Left,
                                                2 => Button::Middle,
                                                3 => Button::Right,
                                                _ => {
                                                    eprintln!("Invalid button value: {}", data[1]);
                                                    continue;
                                                }
                                            };
                                            if let Err(e) = ws_controller.click(button) {
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
                                Ok(_) => {}
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
}
