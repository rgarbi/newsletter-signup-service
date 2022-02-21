use actix_web::{web, HttpResponse, Responder};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Subscriber {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

pub async fn subscriber(subscriber: web::Form<Subscriber>) -> impl Responder {
    println!(
        "{}, {}, {}",
        subscriber.email_address, subscriber.first_name, subscriber.last_name
    );
    HttpResponse::Ok()
}
