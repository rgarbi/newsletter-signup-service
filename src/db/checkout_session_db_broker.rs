use std::str::FromStr;

use crate::domain::checkout_models::{CheckoutSession, CheckoutSessionState};
use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::subscription_models::{NewSubscription, OverTheWireCreateSubscription};

#[tracing::instrument(
    name = "Saving a new checkout session",
    skip(user_id, price_lookup_key, subscription, stripe_session_id, pool)
)]
pub async fn insert_checkout_session(
    user_id: String,
    price_lookup_key: String,
    subscription: NewSubscription,
    stripe_session_id: String,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let checkout_session = CheckoutSession {
        id: Uuid::new_v4(),
        user_id,
        session_state: CheckoutSessionState::Created,
        created_at: Utc::now(),
        price_lookup_key,
        subscription: json!(subscription),
        stripe_session_id,
    };

    sqlx::query!(
        r#"INSERT INTO checkout_session (
            id, 
            user_id, 
            session_state, 
            created_at, 
            price_lookup_key,
            stripe_session_id,
            subscription
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        checkout_session.id,
        checkout_session.user_id,
        checkout_session.session_state.as_str(),
        checkout_session.created_at,
        checkout_session.price_lookup_key,
        checkout_session.stripe_session_id,
        checkout_session.subscription,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Get checkout sessions by stripe session id",
    skip(stripe_session_id, pool)
)]
pub async fn retrieve_checkout_session_by_stripe_session_id(
    stripe_session_id: &String,
    pool: &PgPool,
) -> Result<CheckoutSession, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT
            id, 
            user_id, 
            session_state, 
            created_at, 
            price_lookup_key,
            stripe_session_id,
            subscription
            FROM checkout_session WHERE stripe_session_id = $1"#,
        stripe_session_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    let checkout_session = CheckoutSession {
        id: result.id,
        user_id: result.user_id,
        session_state: CheckoutSessionState::from_str(&result.session_state).unwrap(),
        created_at: result.created_at,
        price_lookup_key: result.price_lookup_key,
        stripe_session_id: result.stripe_session_id,
        subscription: result.subscription,
    };

    Ok(checkout_session)
}

#[tracing::instrument(
    name = "Cancel checkout session by stripe session id",
    skip(stripe_session_id, pool)
)]
pub async fn cancel_checkout_session_by_stripe_session_id(
    stripe_session_id: String,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE checkout_session
            SET session_state = $1
            WHERE stripe_session_id = $2"#,
        CheckoutSessionState::Cancelled.as_str(),
        stripe_session_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Mark checkout session as successful by stripe session id",
    skip(stripe_session_id, transaction)
)]
pub async fn set_checkout_session_state_to_success_by_stripe_session_id(
    stripe_session_id: &String,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE checkout_session
            SET session_state = $1
            WHERE stripe_session_id = $2"#,
        CheckoutSessionState::CompletedSuccessfully.as_str(),
        stripe_session_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
