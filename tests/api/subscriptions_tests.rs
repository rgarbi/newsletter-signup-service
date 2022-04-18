use uuid::Uuid;

use newsletter_signup_service::auth::token::generate_token;
use newsletter_signup_service::domain::subscription_models::{
    OverTheWireCreateSubscription, OverTheWireSubscription, SubscriptionType,
};

use crate::helper::{generate_over_the_wire_create_subscription, spawn_app};

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let subscriber = app.store_subscriber(Option::None).await;
    let test_cases = vec![
        (
            OverTheWireCreateSubscription {
                subscriber_id: subscriber.id.clone(),
                subscription_type: SubscriptionType::Digital,
                subscription_state: Uuid::new_v4().to_string(),
                subscription_name: String::from(""),
                subscription_city: Uuid::new_v4().to_string(),
                subscription_email_address: Uuid::new_v4().to_string(),
                subscription_postal_code: Uuid::new_v4().to_string(),
                subscription_mailing_address_line_2: Option::from(Uuid::new_v4().to_string()),
                subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
            },
            "empty name",
        ),
        (
            OverTheWireCreateSubscription {
                subscriber_id: subscriber.id.clone(),
                subscription_type: SubscriptionType::Digital,
                subscription_state: Uuid::new_v4().to_string(),
                subscription_name: Uuid::new_v4().to_string(),
                subscription_city: Uuid::new_v4().to_string(),
                subscription_email_address: String::from(""),
                subscription_postal_code: Uuid::new_v4().to_string(),
                subscription_mailing_address_line_2: Option::from(Uuid::new_v4().to_string()),
                subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
            },
            "empty email",
        ),
        (
            OverTheWireCreateSubscription {
                subscriber_id: subscriber.id.clone(),
                subscription_type: SubscriptionType::Digital,
                subscription_state: Uuid::new_v4().to_string(),
                subscription_name: Uuid::new_v4().to_string(),
                subscription_city: Uuid::new_v4().to_string(),
                subscription_email_address: Uuid::new_v4().to_string(),
                subscription_postal_code: Uuid::new_v4().to_string(),
                subscription_mailing_address_line_2: Option::from(Uuid::new_v4().to_string()),
                subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
            },
            "invalid email",
        ),
    ];
    for (body, description) in test_cases {
        // Act
        let response = app
            .post_subscription(body.to_json(), generate_token(subscriber.user_id.clone()))
            .await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let body = generate_over_the_wire_create_subscription(subscriber.id.clone());
    let response = app
        .post_subscription(body.to_json(), generate_token(subscriber.user_id.clone()))
        .await;

    assert_eq!(200, response.status().as_u16());

    let saved =
        sqlx::query!("SELECT subscription_name, subscription_postal_code, id FROM subscriptions")
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.subscription_name, body.subscription_name);
}

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_one() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;
    let body = generate_over_the_wire_create_subscription(subscriber.id.clone());
    let response = app
        .post_subscription(body.to_json(), generate_token(subscriber.user_id.clone()))
        .await;
    assert_eq!(200, response.status().as_u16());

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
async fn get_subscriptions_by_subscriber_id_not_found_returns_empty_list() {
    let app = spawn_app().await;

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(
            Uuid::new_v4().to_string(),
            generate_token(Uuid::new_v4().to_string()),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let response_body = subscriptions_response.text().await.unwrap();
    let subscriptions: Vec<OverTheWireSubscription> =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(0, subscriptions.len())
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
        let body = generate_over_the_wire_create_subscription(subscriber.id.clone());
        let response = app
            .post_subscription(body.to_json(), generate_token(subscriber.user_id.clone()))
            .await;
        assert_eq!(200, response.status().as_u16());
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
async fn get_subscriptions_by_id() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;
    let body = generate_over_the_wire_create_subscription(subscriber.id.clone());
    let response = app
        .post_subscription(body.to_json(), generate_token(subscriber.user_id.clone()))
        .await;
    assert_eq!(200, response.status().as_u16());

    let response_body = response.text().await.unwrap();
    let saved_subscription: OverTheWireSubscription =
        serde_json::from_str(response_body.as_str()).unwrap();

    let subscriptions_response = app
        .get_subscription_by_id(
            saved_subscription.id.to_string(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let saved_subscription_response_body = subscriptions_response.text().await.unwrap();
    let subscription: OverTheWireSubscription =
        serde_json::from_str(saved_subscription_response_body.as_str()).unwrap();

    assert_eq!(
        body.subscriber_id.to_string(),
        subscription.subscriber_id.to_string()
    )
}
