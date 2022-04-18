use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::subscription_history_models::{HistoryEventType, SubscriptionHistoryEvent};
use crate::domain::subscription_models::OverTheWireSubscription;

#[tracing::instrument(
    name = "Saving a subscription history event",
    skip(subscription, subscription_change_event_type, pool)
)]
pub async fn insert_subscription_history_event(
    subscription: OverTheWireSubscription,
    subscription_change_event_type: HistoryEventType,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let event = SubscriptionHistoryEvent {
        id: Uuid::new_v4(),
        subscription_id: subscription.id,
        subscription_change_event_type,
        subscription_change_event_date: Utc::now(),
        subscription: json!(subscription),
    };

    sqlx::query!(
        r#"INSERT INTO subscription_event_history (
            id, 
            subscription_id, 
            subscription_change_event_type, 
            subscription_change_event_date, 
            subscription
            ) VALUES ($1, $2, $3, $4, $5)"#,
        event.id,
        event.subscription_id,
        event.subscription_change_event_type.as_str(),
        event.subscription_change_event_date,
        event.subscription
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
pub async fn retrieve_subscription_events_by_subscription_id(
    id: Uuid,
    pool: &PgPool,
) -> Result<Vec<SubscriptionHistoryEvent>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            id, 
            subscription_id, 
            subscription_change_event_date, 
            subscription_change_event_type, 
            subscription
            FROM subscription_event_history WHERE subscription_id = $1"#,
        id
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
                row.subscription_change_event_type,
            ),
            subscription: row.subscription,
        })
    }
    Ok(events)
}
