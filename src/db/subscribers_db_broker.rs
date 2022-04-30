use std::fmt::{Debug, Display};

use actix_web::ResponseError;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::subscriber_models::{NewSubscriber, OverTheWireSubscriber};

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, transaction)
)]
pub async fn insert_subscriber(
    subscriber: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), StoreSubscriberError> {
    sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, name, user_id) VALUES ($1, $2, $3, $4) ON CONFLICT (email_address) DO NOTHING"#,
        Uuid::new_v4(),
        subscriber.email_address.as_ref(),
        subscriber.name.as_ref(),
        subscriber.user_id,
    ).execute(transaction).await.map_err(|e| {
        let err = StoreSubscriberError(e);
        tracing::error!("{:?}", err);
        err

    })?;

    Ok(())
}

#[tracing::instrument(name = "Set Stripe Customer ID", skip(id, stripe_customer_id, pool))]
pub async fn set_stripe_customer_id(
    id: &Uuid,
    stripe_customer_id: &str,
    pool: &PgPool,
) -> Result<(), StoreSubscriberError> {
    sqlx::query!(
        r#"UPDATE subscribers
            SET stripe_customer_id = $1
            WHERE id = $2"#,
        stripe_customer_id,
        id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
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
        r#"SELECT id, email_address, name, user_id, stripe_customer_id FROM subscribers WHERE email_address = $1"#,
        email_address
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(OverTheWireSubscriber {
        id: result.id,
        name: result.name,
        email_address: result.email_address,
        user_id: result.user_id,
        stripe_customer_id: result.stripe_customer_id,
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
        r#"SELECT 
            id, 
            email_address, 
            name, user_id, 
            stripe_customer_id 
          FROM subscribers WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(OverTheWireSubscriber {
        id: result.id,
        name: result.name,
        email_address: result.email_address,
        user_id: result.user_id,
        stripe_customer_id: result.stripe_customer_id,
    })
}

#[tracing::instrument(
    name = "Retrieving a subscriber by user id from the database",
    skip(user_id, pool)
)]
pub async fn retrieve_subscriber_by_user_id(
    user_id: &str,
    pool: &PgPool,
) -> Result<OverTheWireSubscriber, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id, email_address, name, user_id, stripe_customer_id FROM subscribers WHERE user_id = $1"#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(OverTheWireSubscriber {
        id: result.id,
        name: result.name,
        email_address: result.email_address,
        user_id: result.user_id,
        stripe_customer_id: result.stripe_customer_id,
    })
}

#[tracing::instrument(
    name = "Retrieving a subscriber by user id and email address from the database",
    skip(user_id, email_address, pool)
)]
pub async fn retrieve_subscriber_by_user_id_and_email_address(
    user_id: &str,
    email_address: &str,
    pool: &PgPool,
) -> Result<OverTheWireSubscriber, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id, email_address, name, user_id, stripe_customer_id FROM subscribers WHERE user_id = $1 AND email_address = $2"#,
        user_id,
        email_address
    )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(OverTheWireSubscriber {
        id: result.id,
        name: result.name,
        email_address: result.email_address,
        user_id: result.user_id,
        stripe_customer_id: result.stripe_customer_id,
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
