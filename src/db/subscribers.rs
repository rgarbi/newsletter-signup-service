use std::fmt::{Debug, Display};

use actix_web::ResponseError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::new_subscriber::{NewSubscriber, OverTheWireSubscriber};

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, pool)
)]
pub async fn insert_subscriber(
    subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), StoreSubscriberError> {
    sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, first_name, last_name, user_id) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (email_address) DO NOTHING"#,
        Uuid::new_v4(),
        subscriber.email_address.as_ref(),
        subscriber.first_name.as_ref(),
        subscriber.last_name.as_ref(),
        subscriber.user_id,
    ).execute(pool).await.map_err(|e| {
        let err = StoreSubscriberError(e);
        tracing::error!("{:?}", err);
        err

    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Retrieving a subscriber by email address from the database",
    skip(email_address, pool)
)]
pub async fn retrieve_subscriber_by_email(
    email_address: String,
    pool: &PgPool,
) -> Result<OverTheWireSubscriber, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id, email_address, first_name, last_name, user_id FROM subscribers WHERE email_address = $1"#,
        email_address
    ).fetch_one(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(OverTheWireSubscriber {
        id: result.id.to_string(),
        last_name: result.last_name,
        email_address: result.email_address,
        first_name: result.first_name,
        user_id: result.user_id,
    })
}

#[tracing::instrument(
    name = "Retrieving a subscriber by id from the database",
    skip(id, pool)
)]
pub async fn retrieve_subscriber_by_id(
    id: Uuid,
    pool: &PgPool,
) -> Result<OverTheWireSubscriber, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id, email_address, first_name, last_name, user_id FROM subscribers WHERE id = $1"#,
        id
    )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(OverTheWireSubscriber {
        id: result.id.to_string(),
        last_name: result.last_name,
        email_address: result.email_address,
        first_name: result.first_name,
        user_id: result.user_id,
    })
}

#[derive(Debug)]
pub struct StoreSubscriberError(sqlx::Error);

impl Display for StoreSubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscriber."
        )
    }
}

impl ResponseError for StoreSubscriberError {}
