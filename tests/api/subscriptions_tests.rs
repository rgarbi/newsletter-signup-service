use claim::assert_ok;
use uuid::Uuid;

use newsletter_signup_service::auth::token::generate_token;
use newsletter_signup_service::db::subscriptions_db_broker::insert_subscription;
use newsletter_signup_service::domain::subscription_models::{
    NewSubscription, OverTheWireCreateSubscription, OverTheWireSubscription,
};
use newsletter_signup_service::domain::user_models::UserGroup;

use crate::helper::{
    generate_over_the_wire_create_subscription, mock_cancel_stripe_subscription, spawn_app, TestApp,
};

#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;

    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

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

    let subscriber = app.store_subscriber(None).await;
    let _stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(
            subscriber.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
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
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(400, subscriptions_response.status().as_u16());
}

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_with_mismatched_user_ids_not_authorized() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let _stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(
            subscriber.id.to_string(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(401, subscriptions_response.status().as_u16());
}

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_subscription_does_not_exist() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    sqlx::query!("DROP TABLE subscriptions")
        .execute(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(
            subscriber.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(404, subscriptions_response.status().as_u16());
}

#[tokio::test]
async fn get_subscription_by_id_not_found() {
    let app = spawn_app().await;

    let subscriptions_response = app
        .get_subscription_by_id(
            Uuid::new_v4().to_string(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(404, subscriptions_response.status().as_u16());
}

#[tokio::test]
async fn get_subscription_by_id_subscriber_not_found_not_found() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    sqlx::query!("ALTER TABLE subscriptions DROP CONSTRAINT subscriptions_subscriber_id_fkey")
        .execute(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    sqlx::query!("TRUNCATE TABLE subscribers")
        .execute(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(400, subscriptions_response.status().as_u16());
}

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_many() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let expected = 100;

    for _ in 0..expected {
        store_subscription(subscriber.id.to_string(), None, &app).await;
    }

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(
            subscriber.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
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

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
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

#[tokio::test]
async fn cancel_subscription_by_id() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    mock_cancel_stripe_subscription(
        &app.stripe_server,
        stored_subscription.stripe_subscription_id.clone(),
    )
    .await;

    let cancel_subscription_response = app
        .cancel_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, cancel_subscription_response.status().as_u16());
}

#[tokio::test]
async fn cancel_subscription_by_id_not_authorized() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let cancel_subscription_response = app
        .cancel_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(401, cancel_subscription_response.status().as_u16());
}

#[tokio::test]
async fn cancel_subscription_by_id_not_found() {
    let app = spawn_app().await;

    let cancel_subscription_response = app
        .cancel_subscription_by_id(
            Uuid::new_v4().to_string(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(404, cancel_subscription_response.status().as_u16());
}

#[tokio::test]
async fn cancel_subscription_by_id_2x() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    mock_cancel_stripe_subscription(
        &app.stripe_server,
        stored_subscription.stripe_subscription_id.clone(),
    )
    .await;

    let mut cancel_subscription_response = app
        .cancel_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, cancel_subscription_response.status().as_u16());
    cancel_subscription_response = app
        .cancel_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, cancel_subscription_response.status().as_u16());
}

#[tokio::test]
async fn update_subscription() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let update_subscription_response = app
        .update_subscription_by_id(
            stored_subscription.id.to_string(),
            stored_subscription.to_json(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, update_subscription_response.status().as_u16());
}

#[tokio::test]
async fn update_subscription_not_found() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let update_subscription_response = app
        .update_subscription_by_id(
            Uuid::new_v4().to_string(),
            stored_subscription.to_json(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(404, update_subscription_response.status().as_u16());
}

#[tokio::test]
async fn update_subscription_not_authorized() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let update_subscription_response = app
        .update_subscription_by_id(
            stored_subscription.id.to_string(),
            stored_subscription.to_json(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(401, update_subscription_response.status().as_u16());
}

#[tokio::test]
async fn update_subscription_bad_request() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(None).await;
    let stored_subscription = store_subscription(subscriber.id.to_string(), None, &app).await;

    let subscriptions_response = app
        .get_subscription_by_id(
            stored_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let mut bad_email_subscription = stored_subscription.clone();
    bad_email_subscription.subscription_email_address = "a bad email address".to_string();
    let update_subscription_bad_email_response = app
        .update_subscription_by_id(
            stored_subscription.id.to_string(),
            bad_email_subscription.to_json(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(
        400,
        update_subscription_bad_email_response.status().as_u16()
    );

    let mut bad_name_subscription = stored_subscription.clone();
    bad_name_subscription.subscription_name = "      ".to_string();
    let update_subscription_bad_name_response = app
        .update_subscription_by_id(
            stored_subscription.id.to_string(),
            bad_name_subscription.to_json(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(400, update_subscription_bad_name_response.status().as_u16());

    let mut bad_id_subscription = stored_subscription.clone();
    bad_id_subscription.id = Uuid::new_v4();
    let update_subscription_bad_id_response = app
        .update_subscription_by_id(
            stored_subscription.id.to_string(),
            bad_id_subscription.to_json(),
            generate_token(subscriber.user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(400, update_subscription_bad_id_response.status().as_u16());
}

async fn store_subscription(
    subscriber_id: String,
    subscription: Option<OverTheWireCreateSubscription>,
    app: &TestApp,
) -> OverTheWireSubscription {
    let over_the_wire_create_subscription = subscription
        .unwrap_or_else(|| generate_over_the_wire_create_subscription(subscriber_id, None));

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
