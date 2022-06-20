use chrono::Utc;
use claim::{assert_err, assert_ok};
use fake::Faker;
use secrecy::Secret;
use newsletter_signup_service::background::new_subscription_notifier::notify_of_new_subscription;
use newsletter_signup_service::db::subscribers_db_broker::{
    insert_subscriber, retrieve_subscriber_by_user_id,
};
use newsletter_signup_service::db::subscriptions_db_broker::{
    insert_subscription, retrieve_all_subscriptions, retrieve_subscription_by_subscription_id,
    update_subscription_by_subscription_id,
};
use newsletter_signup_service::domain::subscriber_models::NewSubscriber;
use newsletter_signup_service::domain::subscription_models::{
    OverTheWireSubscription, SubscriptionType,
};
use uuid::Uuid;
use newsletter_signup_service::email_client::EmailClient;

use crate::helper::{
    generate_new_subscription, generate_over_the_wire_subscriber, spawn_app, store_subscription,
};

#[tokio::test]
async fn notify_of_new_subscriber_works() {
    let app = spawn_app().await;
    let mock_email_client = app.email_server

    let subscriber = generate_over_the_wire_subscriber();
    let new_subscriber: NewSubscriber = subscriber.clone().try_into().unwrap();
    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    let result = insert_subscriber(&new_subscriber, &mut transaction).await;
    assert_ok!(result);
    assert_ok!(transaction.commit().await);

    let stored_subscriber =
        retrieve_subscriber_by_user_id(subscriber.user_id.as_str(), &app.db_pool)
            .await
            .unwrap();
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    notify_of_new_subscription(stored_subscription.id.clone())
}

fn email_client(base_url: String) -> EmailClient {
    EmailClient::new(
        base_url,
        email(),
        Secret::new(Faker.fake()),
        std::time::Duration::from_millis(200),
    )
}
