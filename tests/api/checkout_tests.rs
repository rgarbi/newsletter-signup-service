use uuid::Uuid;
use newsletter_signup_service::auth::token::LoginResponse;
use newsletter_signup_service::domain::checkout_models::{CreateCheckoutSession};
use newsletter_signup_service::domain::subscriber_models::OverTheWireSubscriber;
use newsletter_signup_service::domain::subscription_models::OverTheWireSubscription;
use newsletter_signup_service::util::generate_random_token;
use crate::helper::{generate_over_the_wire_create_subscription, generate_signup, mock_create_checkout_session, mock_get_stripe_session, mock_stripe_create_customer, mock_stripe_price_lookup, spawn_app};



#[tokio::test]
async fn create_checkout_session_not_authorized() {
    let app = spawn_app().await;

    let subscription = generate_over_the_wire_create_subscription(Uuid::new_v4().to_string());
    let create_checkout_session = CreateCheckoutSession {
        price_lookup_key: Uuid::new_v4().to_string(),
        subscription
    };

    let checkout_response = app.post_checkout(create_checkout_session.to_json(), Uuid::new_v4().to_string(), generate_random_token()).await;
    assert_eq!(401, checkout_response.status().as_u16());
}

#[tokio::test]
async fn create_checkout_session_subscriber_not_found() {
    let app = spawn_app().await;
    //SIGN UP
    let sign_up = generate_signup();
    let sign_up_response= app.user_signup(sign_up.to_json()).await;
    assert_eq!(&200, &sign_up_response.status().as_u16());
    let sign_up_response_body = sign_up_response.text().await.unwrap();
    let login: LoginResponse = serde_json::from_str(sign_up_response_body.as_str()).unwrap();


    let price_lookup_key = Uuid::new_v4().to_string();
    let subscription = generate_over_the_wire_create_subscription(Uuid::new_v4().to_string());
    let create_checkout_session = CreateCheckoutSession {
        price_lookup_key: price_lookup_key.clone(),
        subscription
    };

    let checkout_response = app.post_checkout(create_checkout_session.to_json(), login.user_id.clone(), login.token.clone()).await;
    assert_eq!(400, checkout_response.status().as_u16());
}

#[tokio::test]
async fn create_checkout_session_stripe_customer_blows_up() {
}


