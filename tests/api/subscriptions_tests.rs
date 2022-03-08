use uuid::Uuid;

use newsletter_signup_service::domain::new_subscription::{
    OverTheWireCreateSubscription, SubscriptionType,
};

use crate::helper::spawn_app;
use crate::helper::store_subscriber;

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

    let body = OverTheWireCreateSubscription {
        subscriber_id: subscriber.id,
        subscription_type: SubscriptionType::Electronic,
        subscription_state: Uuid::new_v4().to_string(),
        subscription_last_name: Uuid::new_v4().to_string(),
        subscription_city: Uuid::new_v4().to_string(),
        subscription_email_address: format!("{}@gmail.com", Uuid::new_v4().to_string()),
        subscription_postal_code: Uuid::new_v4().to_string(),
        subscription_mailing_address_line_2: Option::from(Uuid::new_v4().to_string()),
        subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
        subscription_first_name: Uuid::new_v4().to_string(),
    };
    let response = app.post_subscription(body.to_json()).await;

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT subscription_first_name, subscription_last_name, subscription_postal_code, id FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.subscription_first_name, body.subscription_first_name);
    assert_eq!(saved.subscription_last_name, body.subscription_last_name);
}
