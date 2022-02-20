use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use crate::models::subscriber::Subscriber;

mod models;

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

async fn subscription() -> impl Responder {
    HttpResponse::Ok()
}

async fn subscriber(subscriber: web::Form<Subscriber>) -> impl Responder {
    println!(
        "{}, {}, {}",
        subscriber.email_address, subscriber.first_name, subscriber.last_name
    );
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscription))
            .route("/subscribers", web::post().to(subscriber))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
