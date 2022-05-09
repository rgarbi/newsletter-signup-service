use crate::db::subscription_history_db_broker::insert_subscription_history_event;
use crate::db::subscriptions_db_broker::retrieve_subscription_by_subscription_id;
use crate::domain::subscription_history_models::HistoryEventType;
use sqlx::PgPool;
use uuid::Uuid;

pub fn store_subscription_history_event(
    subscription_id: Uuid,
    subscription_change_event_type: HistoryEventType,
    pool: &PgPool,
) {
    let new_pool = pool.clone();
    tokio::spawn(async move {
        if let Ok(subscription) =
            retrieve_subscription_by_subscription_id(subscription_id, &new_pool).await
        {
            if let Ok(_) = insert_subscription_history_event(
                subscription,
                subscription_change_event_type,
                &new_pool,
            )
            .await
            {}
        }
    });
}
