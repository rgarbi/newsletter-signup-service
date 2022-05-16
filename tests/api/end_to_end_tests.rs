use uuid::Uuid;
use newsletter_signup_service::auth::token::LoginResponse;
use newsletter_signup_service::domain::checkout_models::{CreateCheckoutSession};
use newsletter_signup_service::domain::subscriber_models::OverTheWireSubscriber;
use newsletter_signup_service::domain::subscription_models::OverTheWireSubscription;
use crate::helper::{generate_over_the_wire_create_subscription, generate_signup, mock_create_checkout_session, mock_get_stripe_session, mock_stripe_create_customer, mock_stripe_price_lookup, spawn_app};



/*
    1. Create a user account
    2. Sign up for a subscription.
    3. Complete Payment.
*/
#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    //SIGN UP
    let login: LoginResponse = app.sign_up().await;

    //GET SUBSCRIBER BY USER ID
    let subscriber: OverTheWireSubscriber = app.get_subscriber_by_user_id(login.user_id.clone(), login.token.clone()).await;

    //SUBSCRIBE!
    let price_lookup_key = Uuid::new_v4().to_string();
    let stripe_session_id = Uuid::new_v4().to_string();
    mock_stripe_create_customer(&app.stripe_server, subscriber.email_address.clone()).await;
    mock_stripe_price_lookup(&app.stripe_server, price_lookup_key.clone()).await;
    mock_create_checkout_session(&app.stripe_server, stripe_session_id.clone()).await;
    mock_get_stripe_session(&app.stripe_server, stripe_session_id.clone()).await;

    let subscription = generate_over_the_wire_create_subscription(subscriber.id.to_string().clone());
    let create_checkout_session = CreateCheckoutSession {
        price_lookup_key: price_lookup_key.clone(),
        subscription
    };

    let checkout_response = app.post_checkout(create_checkout_session.to_json(), login.user_id.clone(), login.token.clone()).await;
    assert_eq!(&200, &checkout_response.status().as_u16());

    //COMPLETE SUBSCRIPTION
    let complete_session_response = app.post_complete_session(login.user_id.clone(), stripe_session_id.clone(), login.token.clone()).await;
    assert_eq!(&200, &complete_session_response.status().as_u16());

    //ASSERT WE HAVE 1 SUBSCRIPTION IN THE LIST!
    let subscription_list_response = app.get_subscriptions_by_subscriber_id(subscriber.id.to_string().clone(), login.token.clone()).await;
    assert_eq!(&200, &subscription_list_response.status().as_u16());
    let subscription_list_response_body = subscription_list_response.text().await.unwrap();
    let subscription_list: Vec<OverTheWireSubscription> = serde_json::from_str(subscription_list_response_body.as_str()).unwrap();
    assert_eq!(1, subscription_list.len());
}