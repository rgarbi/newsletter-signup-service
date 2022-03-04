use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::new_subscriber::{NewSubscriber, OverTheWireCreateSubscriber};
use crate::domain::subscriber_name::SubscriberName;

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
    subscriber: web::Form<OverTheWireCreateSubscriber>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let new_subscriber = NewSubscriber {
        first_name: SubscriberName::parse(subscriber.0.first_name)
            .expect("Unable to validate the first name"),
        last_name: SubscriberName::parse(subscriber.0.last_name)
            .expect("Unable to validate the last name"),
        email_address: subscriber.0.email_address,
    };

    match insert_subscriber(&new_subscriber, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, pool)
)]
pub async fn insert_subscriber(
    subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, first_name, last_name) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        subscriber.email_address,
        subscriber.first_name.as_ref(),
        subscriber.last_name.as_ref()
    ).execute(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
