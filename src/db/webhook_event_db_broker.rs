use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::domain::webhook_event::WebhookEvent;

#[tracing::instrument(
    name = "Insert a webhook event",
    skip(webhook_event, pool)
)]
pub async fn insert_webhook_event(
    webhook_event: &WebhookEvent,
    pool: &PgPool,
) -> Result<Uuid, Error> {
    sqlx::query!(
        r#"INSERT 
            INTO webhook_events (id, event_text, sent_on, processed)
            VALUES ($1, $2, $3, $4)"#,
        webhook_event.id,
        webhook_event.event_text,
        webhook_event.sent_on,
        webhook_event.processed,
    )
    .execute(pool)
    .await
    .map_err(|e: Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(webhook_event.id)
}

#[tracing::instrument(
    name = "Set a webhook event to processed",
    skip(id, pool)
)]
pub async fn set_webhook_event_to_processed(
    id: Uuid,
    pool: &PgPool,
) -> Result<Uuid, Error> {
    sqlx::query!(
        r#"UPDATE webhook_events
            SET processed = true
            WHERE id = $1"#,
        id
    )
        .execute(pool)
        .await
        .map_err(|e: Error| {
            tracing::error!("{:?}", e);
            e
        })?;

    Ok(id)
}

#[tracing::instrument(
    name = "Get all unprocessed webhook events",
    skip(pool)
)]
pub async fn get_unprocessed_webhook_events(
    pool: &PgPool,
) -> Result<Vec<WebhookEvent>, Error> {
    let results = sqlx::query!(
        r#"SELECT
            id, event_text, sent_on, processed
            FROM webhook_events
            WHERE processed = false"#)
        .fetch_all(pool)
        .await
        .map_err(|e: Error| {
            tracing::error!("{:?}", e);
            e
        })?;

    let mut events: Vec<WebhookEvent> = Vec::new();

    for event in results {
        events.push(WebhookEvent {
            id: event.id,
            event_text: event.event_text,
            sent_on: event.sent_on,
            processed: event.processed,
        });
    }

    Ok(events)
}
