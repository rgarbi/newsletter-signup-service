use crate::models::subscriber::Subscriber;
use actix_web::{web, HttpResponse, Responder};

pub async fn subscriber(subscriber: web::Form<Subscriber>) -> impl Responder {
    println!(
        "{}, {}, {}",
        subscriber.email_address, subscriber.first_name, subscriber.last_name
    );
    HttpResponse::Ok()
}
