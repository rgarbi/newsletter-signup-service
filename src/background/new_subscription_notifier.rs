use crate::db::subscriptions_db_broker::retrieve_subscription_by_subscription_id;
use crate::email_client::EmailClient;
use sqlx::PgPool;
use uuid::Uuid;

pub fn notify_of_new_subscription(subscription_id: Uuid, email_client: EmailClient, pool: &PgPool) {
    let new_pool = pool.clone();
    tokio::spawn(async move {
        if let Ok(subscription) =
            retrieve_subscription_by_subscription_id(subscription_id, &new_pool).await
        {
            email_client.send_email()
        }
    });
}
