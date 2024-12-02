mod http_server;
mod mouse_controller;
mod websocket_server;

use http_server::start_http_server;
use local_ip_address::local_ip;
use mouse_controller::MouseController;
use qrcode::QrCode;
use std::sync::Arc;
use websocket_server::start_websocket_server;

fn print_qr_code(text: &str) {
    let code = QrCode::new(text).unwrap();
    let string = code
        .render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1) // Further adjusted dimensions to make the QR code smaller
        .build();
    println!("\n{}\n", string);
}
fn main() {
    let controller = Arc::new(MouseController::new());
    start_websocket_server(Arc::clone(&controller));

    let local_ip = local_ip().unwrap();
    let url = format!("http://{}:8081", local_ip);
    println!("Scan QR code or go to: {}", url);
    print_qr_code(&url);

    // Run HTTP server on main thread
    start_http_server();
}
