use std::str::FromStr;

use crate::domain::checkout_models::{CheckoutSession, CheckoutSessionState};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::subscription_history_models::{HistoryEventType, SubscriptionHistoryEvent};
use crate::domain::subscription_models::{OverTheWireCreateSubscription, OverTheWireSubscription};

#[tracing::instrument(
    name = "Saving a new checkout session",
    skip(user_id, price_lookup_key, subscription, stripe_session_id, pool)
)]
pub async fn insert_checkout_session(
    user_id: String,
    price_lookup_key: String,
    subscription: OverTheWireCreateSubscription,
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
    name = "Get all subscription events by subscription id",
    skip(id, pool)
)]
pub async fn retrieve_checkout_session_by_stripe_session_id(
    stripe_session_id: String,
    pool: &PgPool,
) -> Result<Vec<SubscriptionHistoryEvent>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            id, 
            user_id, 
            session_state, 
            created_at, 
            price_lookup_key,
            stripe_session_id,
            subscription,
            FROM checkout_session WHERE stripe_session_id = $1"#,
        stripe_session_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    let mut events: Vec<SubscriptionHistoryEvent> = Vec::new();

    for row in rows {
        events.push(SubscriptionHistoryEvent {
            id: row.id,
            subscription_id: row.subscription_id,
            subscription_change_event_date: row.subscription_change_event_date,
            subscription_change_event_type: HistoryEventType::from_str(
                &row.subscription_change_event_type,
            )
            .unwrap(),
            subscription: row.subscription,
        })
    }
    Ok(events)
}
