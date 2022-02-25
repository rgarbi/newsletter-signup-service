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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscriber, pool),
    fields(
        subscriber_email = %subscriber.email_address,
    )
)]
pub async fn post_subscriber(
    subscriber: web::Form<Subscriber>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    match insert_subscriber(&subscriber, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, pool)
)]
pub async fn insert_subscriber(subscriber: &Subscriber, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, first_name, last_name) VALUES ($1, $2, $3, $4)"#,
        Uuid::from_str(&*subscriber.id).expect("Unable to parse the UUID"),
        subscriber.email_address,
        subscriber.first_name,
        subscriber.last_name
    ).execute(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
