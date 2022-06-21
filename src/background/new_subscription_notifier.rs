use crate::configuration::get_configuration;
use crate::db::subscriptions_db_broker::retrieve_subscription_by_subscription_id;
use crate::domain::valid_email::ValidEmail;
use crate::email_client::EmailClient;
use sqlx::PgPool;
use uuid::Uuid;

pub fn notify_of_new_subscription(subscription_id: Uuid, email_client: EmailClient, pool: &PgPool) {
    let new_pool = pool.clone();
    tokio::spawn(async move {
        notify_subscriber(subscription_id, email_client, &new_pool).await;
    });
}

pub async fn notify_subscriber(subscription_id: Uuid, email_client: EmailClient, pool: &PgPool) {
    if let Ok(subscription) = retrieve_subscription_by_subscription_id(subscription_id, pool).await
    {
        let recipients = get_configuration()
            .unwrap()
            .application_feature_settings
            .subscription_notification_addresses
            .iter()
            .map(|recipient| ValidEmail::parse(recipient.clone()).unwrap())
            .collect::<Vec<ValidEmail>>();

        let _ = email_client
            .send_email(
                recipients,
                "New Subscription",
                "Something",
                subscription.to_json().as_str(),
            )
            .await
            .is_ok();
    }
}
