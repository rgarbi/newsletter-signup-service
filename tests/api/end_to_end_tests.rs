

use crate::helper::{spawn_app, TestApp};



/*
    1. Create a user account
    2. Sign up for a subscription.
    3. Complete Payment.
*/
#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let stored_subscription = store_subscription(subscriber.id.to_string(), &app).await;

    let saved =
        sqlx::query!("SELECT subscription_name, subscription_postal_code, id FROM subscriptions")
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved subscription.");
    assert_eq!(
        saved.subscription_name,
        stored_subscription.subscription_name
    );
}
