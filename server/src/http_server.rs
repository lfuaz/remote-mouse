use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use include_dir::{include_dir, Dir};
use mime_guess::from_path;
use std::path::Path;

static DIST_DIR: Dir = include_dir!("./dist");

async fn serve_file(path: web::Path<String>, _req: HttpRequest) -> impl Responder {
    let file_path = path.into_inner();

    if let Some(file) = DIST_DIR.get_file(&file_path) {
        let mime_type = from_path(file.path()).first_or_octet_stream();
        HttpResponse::Ok()
            .content_type(mime_type.as_ref())
            .body(file.contents().to_vec())
    } else {
        if Path::new(&file_path).extension().is_none() {
            if let Some(index_file) = DIST_DIR.get_file("index.html") {
                let mime_type = from_path(index_file.path()).first_or_octet_stream();
                HttpResponse::Ok()
                    .content_type(mime_type.as_ref())
                    .body(index_file.contents())
            } else {
                println!("index.html not found in DIST_DIR");
                HttpResponse::NotFound().finish()
            }
        } else {
            println!("Static asset not found: {}", file_path);
            HttpResponse::NotFound().finish()
        }
    }
}

pub async fn start_http_server() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(serve_file)))
        .bind("0.0.0.0:8081")?
        .run()
        .await
}
