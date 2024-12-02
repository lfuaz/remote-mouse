use include_dir::{include_dir, Dir};
use mime_guess::from_path;
use tiny_http::{Response, Server};

static DIST_DIR: Dir = include_dir!("./dist");

pub fn start_http_server() {
    let server = Server::http("0.0.0.0:8081").unwrap();
    for request in server.incoming_requests() {
        let url_path = request.url().trim_start_matches('/');
        let path = if url_path.is_empty() {
            "index.html"
        } else {
            url_path
        };

        let response = if let Some(file) = DIST_DIR.get_file(path) {
            let mime_type = from_path(path).first_or_octet_stream();
            Response::from_data(file.contents()).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], mime_type.as_ref().as_bytes())
                    .unwrap(),
            )
        } else if let Some(index_file) = DIST_DIR.get_file("index.html") {
            Response::from_data(index_file.contents()).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
            )
        } else {
            Response::from_string("404 Not Found").with_status_code(404)
        };

        let _ = request.respond(response);
    }
}
