mod http_server;
mod mouse_controller;
mod websocket_server;

use http_server::start_http_server;
use local_ip_address::local_ip;
use mouse_controller::MouseController;
use qrcode::QrCode;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::net::TcpListener;
use std::sync::Arc;
use websocket_server::start_websocket_server;

/// Bind to `port`, or the next free port above it.
fn bind_from(port: u16) -> TcpListener {
    (port..=u16::MAX)
        .find_map(|p| TcpListener::bind(("0.0.0.0", p)).ok())
        .expect("No free port available")
}

/// Random per-run key; RandomState is seeded from the OS RNG.
fn random_key() -> String {
    format!("{:016x}", RandomState::new().build_hasher().finish())
}

fn print_qr_code(text: &str) {
    let code = QrCode::new(text).unwrap();
    let string = code
        .render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    println!("\n{}\n", string);
}

fn main() {
    let http = bind_from(8081);
    let ws = bind_from(9001);
    let http_port = http.local_addr().unwrap().port();
    let ws_port = ws.local_addr().unwrap().port();

    let key = random_key();

    start_websocket_server(ws, Arc::new(MouseController::new()), key.clone());

    // The client reads the WebSocket port and access key from the query string.
    let url = format!(
        "http://{}:{}/?ws={}&key={}",
        local_ip().unwrap(),
        http_port,
        ws_port,
        key
    );
    println!("Scan QR code or go to: {}", url);
    print_qr_code(&url);

    start_http_server(http);
}

#[cfg(test)]
mod tests {
    #[test]
    fn bind_from_skips_taken_port() {
        let taken = super::bind_from(40000);
        let port = taken.local_addr().unwrap().port();
        assert!(super::bind_from(port).local_addr().unwrap().port() > port);
    }
}
