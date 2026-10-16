use include_dir::{include_dir, Dir};
use std::net::TcpListener;
use tiny_http::{Header, Response, Server};

static DIST_DIR: Dir = include_dir!("./dist");

pub fn start_http_server(listener: TcpListener) {
    let server = Server::from_listener(listener, None).unwrap();
    for request in server.incoming_requests() {
        let path = request.url().split('?').next().unwrap().trim_start_matches('/');
        // Unknown paths fall back to index.html (SPA).
        let (path, file) = match DIST_DIR.get_file(path) {
            Some(f) => (path, Some(f)),
            None => ("index.html", DIST_DIR.get_file("index.html")),
        };
        let response = match file {
            Some(f) => {
                // dist/ only holds what vite emits; extend when it emits new kinds.
                let mime = match path.rsplit('.').next() {
                    Some("html") => "text/html",
                    Some("js") => "text/javascript",
                    Some("css") => "text/css",
                    Some("svg") => "image/svg+xml",
                    _ => "application/octet-stream",
                };
                Response::from_data(f.contents()).with_header(
                    Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap(),
                )
            }
            None => Response::from_string("404 Not Found").with_status_code(404),
        };
        let _ = request.respond(response);
    }
}
