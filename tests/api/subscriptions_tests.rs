use claim::assert_ok;
use uuid::Uuid;

use newsletter_signup_service::auth::token::generate_token;
use newsletter_signup_service::db::subscriptions_db_broker::insert_subscription;
use newsletter_signup_service::domain::subscription_models::{
    NewSubscription, OverTheWireCreateSubscription, OverTheWireSubscription, SubscriptionType,
};

use crate::helper::{generate_over_the_wire_create_subscription, spawn_app, TestApp};

#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let stored_subscription = store_subscription(subscriber.id.clone(), &app).await;

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

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_one() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;
    let _stored_subscription = store_subscription(subscriber.id.clone(), &app).await;

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(
            subscriber.id.clone(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let response_body = subscriptions_response.text().await.unwrap();
    let subscriptions: Vec<OverTheWireSubscription> =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(1, subscriptions.len())
}

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_not_found_returns_bad_request() {
    let app = spawn_app().await;

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(
            Uuid::new_v4().to_string(),
            generate_token(Uuid::new_v4().to_string()),
        )
        .await;
    assert_eq!(400, subscriptions_response.status().as_u16());
}

#[tokio::test]
async fn get_subscription_by_id_not_found() {
    let app = spawn_app().await;

    let subscriptions_response = app
        .get_subscription_by_id(
            Uuid::new_v4().to_string(),
            generate_token(Uuid::new_v4().to_string()),
        )
        .await;
    assert_eq!(404, subscriptions_response.status().as_u16());
}

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_many() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;
    let expected = 100;

    for _ in 0..expected {
        store_subscription(subscriber.id.clone(), &app).await;
    }

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(
            subscriber.id.clone(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let response_body = subscriptions_response.text().await.unwrap();
    let subscriptions: Vec<OverTheWireSubscription> =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(expected, subscriptions.len())
}

#[tokio::test]
async fn get_subscription_by_id() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;
    let stored_subscription = store_subscription(subscriber.id.clone(), &app).await;

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let saved_subscription_response_body = subscriptions_response.text().await.unwrap();
    let subscription: OverTheWireSubscription =
        serde_json::from_str(saved_subscription_response_body.as_str()).unwrap();

    assert_eq!(
        stored_subscription.subscriber_id.to_string(),
        subscription.subscriber_id.to_string()
    )
}

async fn store_subscription(subscriber_id: String, app: &TestApp) -> OverTheWireSubscription {
    let over_the_wire_create_subscription =
        generate_over_the_wire_create_subscription(subscriber_id);

    let mut transaction_result = app.db_pool.begin().await;
    assert_ok!(&mut transaction_result);
    let mut transaction = transaction_result.unwrap();

    let new_subscription: NewSubscription = over_the_wire_create_subscription.try_into().unwrap();

    let response = insert_subscription(
        new_subscription,
        Uuid::new_v4().to_string(),
        &mut transaction,
    )
    .await;
    assert_ok!(&response);
    assert_ok!(transaction.commit().await);
    response.unwrap()
}
