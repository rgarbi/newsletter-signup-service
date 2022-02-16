use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

mod models;

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
