use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Subscriber {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

pub async fn subscriber(
    subscriber: web::Form<Subscriber>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let result = sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, first_name, last_name) VALUES ($1, $2, $3, $4)"#,
        Uuid::from_str(&*subscriber.id).expect("Unable to parse the UUID"),
        subscriber.email_address,
        subscriber.first_name,
        subscriber.last_name
    ).execute(pool.get_ref()).await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
