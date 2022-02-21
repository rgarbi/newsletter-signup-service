use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Subscriber {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

pub async fn subscriber(
    subscriber: web::Form<Subscriber>,
    connection: web::Data<PgConnection>,
) -> impl Responder {
    sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, first_name, last_name) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        subscriber.email_address,
        subscriber.first_name,
        subscriber.last_name
    ).execute(connection.get_ref()).await;
    HttpResponse::Ok()
}
