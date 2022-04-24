use uuid::Uuid;

use newsletter_signup_service::auth::token::generate_token;

use newsletter_signup_service::domain::checkout_models::CreateCheckoutSession;
use newsletter_signup_service::domain::subscription_models::{
    OverTheWireCreateSubscription, SubscriptionType,
};

use crate::helper::spawn_app;

#[tokio::test]
async fn checkout_returns_a_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let subscriber = app.store_subscriber(Option::None).await;
    let test_cases = vec![
        (
            CreateCheckoutSession {
                price_lookup_key: Uuid::new_v4().to_string(),
                subscription: OverTheWireCreateSubscription {
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
            },
            "empty name",
        ),
        (
            CreateCheckoutSession {
                price_lookup_key: Uuid::new_v4().to_string(),
                subscription: OverTheWireCreateSubscription {
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
            },
            "empty email",
        ),
        (
            CreateCheckoutSession {
                price_lookup_key: Uuid::new_v4().to_string(),
                subscription: OverTheWireCreateSubscription {
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
            },
            "invalid email",
        ),
    ];
    for (body, description) in test_cases {
        // Act
        let response = app
            .post_checkout(
                body.to_json(),
                subscriber.user_id.clone(),
                generate_token(subscriber.user_id.clone()),
            )
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
