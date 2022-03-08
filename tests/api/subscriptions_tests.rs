use uuid::Uuid;

use newsletter_signup_service::domain::new_subscription::{
    OverTheWireCreateSubscription, OverTheWireSubscription, SubscriptionType,
};

use crate::helper::store_subscriber;
use crate::helper::{generate_over_the_wire_subscription, spawn_app};

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let subscriber = store_subscriber(app.clone()).await;
    let test_cases = vec![
        (
            OverTheWireCreateSubscription {
                subscriber_id: subscriber.id.clone(),
                subscription_type: SubscriptionType::Electronic,
                subscription_state: Uuid::new_v4().to_string(),
                subscription_last_name: String::from(""),
                subscription_city: Uuid::new_v4().to_string(),
                subscription_email_address: Uuid::new_v4().to_string(),
                subscription_postal_code: Uuid::new_v4().to_string(),
                subscription_mailing_address_line_2: Option::from(Uuid::new_v4().to_string()),
                subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
                subscription_first_name: String::from(""),
            },
            "empty name",
        ),
        (
            OverTheWireCreateSubscription {
                subscriber_id: subscriber.id.clone(),
                subscription_type: SubscriptionType::Electronic,
                subscription_state: Uuid::new_v4().to_string(),
                subscription_last_name: String::from(""),
                subscription_city: Uuid::new_v4().to_string(),
                subscription_email_address: String::from(""),
                subscription_postal_code: Uuid::new_v4().to_string(),
                subscription_mailing_address_line_2: Option::from(Uuid::new_v4().to_string()),
                subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
                subscription_first_name: String::from(""),
            },
            "empty email",
        ),
        (
            OverTheWireCreateSubscription {
                subscriber_id: subscriber.id.clone(),
                subscription_type: SubscriptionType::Electronic,
                subscription_state: Uuid::new_v4().to_string(),
                subscription_last_name: String::from(""),
                subscription_city: Uuid::new_v4().to_string(),
                subscription_email_address: Uuid::new_v4().to_string(),
                subscription_postal_code: Uuid::new_v4().to_string(),
                subscription_mailing_address_line_2: Option::from(Uuid::new_v4().to_string()),
                subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
                subscription_first_name: String::from(""),
            },
            "invalid email",
        ),
    ];
    for (body, description) in test_cases {
        // Act
        let response = app.post_subscription(body.to_json()).await;
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

    let subscriber = store_subscriber(app.clone()).await;

    let body = generate_over_the_wire_subscription(subscriber.id.clone());
    let response = app.post_subscription(body.to_json()).await;

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT subscription_first_name, subscription_last_name, subscription_postal_code, id FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.subscription_first_name, body.subscription_first_name);
    assert_eq!(saved.subscription_last_name, body.subscription_last_name);
}

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_one() {
    let app = spawn_app().await;

    let subscriber = store_subscriber(app.clone()).await;
    let body = generate_over_the_wire_subscription(subscriber.id.clone());
    let response = app.post_subscription(body.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(subscriber.id.clone())
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let response_body = subscriptions_response.text().await.unwrap();
    let subscriptions: Vec<OverTheWireSubscription> =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(1, subscriptions.len())
}

#[tokio::test]
async fn get_subscriptions_by_subscriber_id_many() {
    let app = spawn_app().await;

    let subscriber = store_subscriber(app.clone()).await;
    let expected = 100;

    for _ in 0..expected {
        let body = generate_over_the_wire_subscription(subscriber.id.clone());
        let response = app.post_subscription(body.to_json()).await;
        assert_eq!(200, response.status().as_u16());
    }

    let subscriptions_response = app
        .get_subscriptions_by_subscriber_id(subscriber.id.clone())
        .await;
    assert_eq!(200, subscriptions_response.status().as_u16());

    let response_body = subscriptions_response.text().await.unwrap();
    let subscriptions: Vec<OverTheWireSubscription> =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(expected, subscriptions.len())
}
