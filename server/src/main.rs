mod http_server;
mod mouse_controller;
mod websocket_server;

use http_server::start_http_server;
use local_ip_address::local_ip;
use mouse_controller::MouseController;
use qrcode::QrCode;
use std::sync::Arc;
use websocket_server::start_websocket_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let controller = Arc::new(MouseController::new());
    start_websocket_server(Arc::clone(&controller));

    let local_ip = local_ip().unwrap();
    let url = format!("http://{}:8081", local_ip);
    println!("Go to website: {}", url);

    let code = QrCode::new(&url).unwrap();
    let qr_code_console = code.render::<qrcode::render::unicode::Dense1x2>().build();
    println!("{}", qr_code_console);

    start_http_server().await
}
