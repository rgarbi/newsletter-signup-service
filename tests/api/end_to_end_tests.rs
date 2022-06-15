use crate::helper::{
    generate_over_the_wire_create_subscription, mock_create_checkout_session,
    mock_get_stripe_session, mock_stripe_create_customer, mock_stripe_price_lookup, spawn_app,
};
use newsletter_signup_service::auth::token::LoginResponse;
use newsletter_signup_service::configuration::get_configuration;
use newsletter_signup_service::domain::checkout_models::CreateCheckoutSession;
use newsletter_signup_service::domain::subscriber_models::OverTheWireSubscriber;
use newsletter_signup_service::domain::subscription_models::{
    OverTheWireSubscription, SubscriptionType,
};
use uuid::Uuid;

/*
    1. Create a user account
    2. Sign up for a subscription.
    3. Complete Payment.
*/
#[tokio::test]
async fn end_to_end_subscribe_test() {
    let app = spawn_app().await;
    let config = get_configuration().unwrap().stripe_client;

    //SIGN UP
    let login: LoginResponse = app.sign_up().await;

    //GET SUBSCRIBER BY USER ID
    let subscriber: OverTheWireSubscriber = app
        .get_subscriber_by_user_id(login.user_id.clone(), login.token.clone())
        .await;

    //SUBSCRIBE!
    let price_lookup_key = config.digital_price_lookup_key;
    let stripe_session_id = Uuid::new_v4().to_string();
    mock_stripe_create_customer(&app.stripe_server, subscriber.email_address.clone()).await;
    mock_stripe_price_lookup(&app.stripe_server, price_lookup_key.clone()).await;
    mock_create_checkout_session(&app.stripe_server, stripe_session_id.clone()).await;
    mock_get_stripe_session(&app.stripe_server, stripe_session_id.clone()).await;

    let subscription = generate_over_the_wire_create_subscription(
        subscriber.id.to_string().clone(),
        Some(SubscriptionType::Digital),
    );
    let create_checkout_session = CreateCheckoutSession {
        price_lookup_key: price_lookup_key.clone(),
        subscription,
    };

    let checkout_response = app
        .post_checkout(
            create_checkout_session.to_json(),
            login.user_id.clone(),
            login.token.clone(),
        )
        .await;
    assert_eq!(&200, &checkout_response.status().as_u16());

    //COMPLETE SUBSCRIPTION
    let complete_session_response = app
        .post_complete_session(
            login.user_id.clone(),
            stripe_session_id.clone(),
            login.token.clone(),
        )
        .await;
    assert_eq!(&200, &complete_session_response.status().as_u16());

    //ASSERT WE HAVE 1 SUBSCRIPTION IN THE LIST!
    let subscription_list_response = app
        .get_subscriptions_by_subscriber_id(subscriber.id.to_string().clone(), login.token.clone())
        .await;
    assert_eq!(&200, &subscription_list_response.status().as_u16());
    let subscription_list_response_body = subscription_list_response.text().await.unwrap();
    let subscription_list: Vec<OverTheWireSubscription> =
        serde_json::from_str(subscription_list_response_body.as_str()).unwrap();
    assert_eq!(1, subscription_list.len());
}

/*
    1. Create a user account
    2. Sign up for a subscription.
    3. Complete Payment.
    4. Sign up for a subscription, again.
    5. Complete Payment
*/
#[tokio::test]
async fn end_to_end_subscribe_test_2x() {
    let app = spawn_app().await;
    let config = get_configuration().unwrap().stripe_client;

    //SIGN UP
    let login: LoginResponse = app.sign_up().await;

    //GET SUBSCRIBER BY USER ID
    let subscriber: OverTheWireSubscriber = app
        .get_subscriber_by_user_id(login.user_id.clone(), login.token.clone())
        .await;

    //SUBSCRIBE!
    let price_lookup_key = config.paper_price_lookup_key;
    let stripe_session_id = Uuid::new_v4().to_string();
    mock_stripe_create_customer(&app.stripe_server, subscriber.email_address.clone()).await;
    mock_stripe_price_lookup(&app.stripe_server, price_lookup_key.clone()).await;
    mock_create_checkout_session(&app.stripe_server, stripe_session_id.clone()).await;
    mock_get_stripe_session(&app.stripe_server, stripe_session_id.clone()).await;

    let subscription = generate_over_the_wire_create_subscription(
        subscriber.id.to_string().clone(),
        Some(SubscriptionType::Paper),
    );
    let create_checkout_session = CreateCheckoutSession {
        price_lookup_key: price_lookup_key.clone(),
        subscription,
    };

    let checkout_response = app
        .post_checkout(
            create_checkout_session.to_json(),
            login.user_id.clone(),
            login.token.clone(),
        )
        .await;
    assert_eq!(&200, &checkout_response.status().as_u16());

    //COMPLETE SUBSCRIPTION
    let complete_session_response = app
        .post_complete_session(
            login.user_id.clone(),
            stripe_session_id.clone(),
            login.token.clone(),
        )
        .await;
    assert_eq!(&200, &complete_session_response.status().as_u16());

    //SUBSCRIBE again!
    let _x = &app.stripe_server.reset().await;
    let stripe_session_id_again = Uuid::new_v4().to_string();
    mock_stripe_price_lookup(&app.stripe_server, price_lookup_key.clone()).await;
    mock_create_checkout_session(&app.stripe_server, stripe_session_id_again.clone()).await;
    mock_get_stripe_session(&app.stripe_server, stripe_session_id_again.clone()).await;

    let subscription_again = generate_over_the_wire_create_subscription(
        subscriber.id.to_string().clone(),
        Some(SubscriptionType::Paper),
    );
    let create_checkout_session_again = CreateCheckoutSession {
        price_lookup_key: price_lookup_key.clone(),
        subscription: subscription_again,
    };

    let checkout_response_again = app
        .post_checkout(
            create_checkout_session_again.to_json(),
            login.user_id.clone(),
            login.token.clone(),
        )
        .await;
    assert_eq!(&200, &checkout_response_again.status().as_u16());

    //COMPLETE SUBSCRIPTION
    let complete_session_response_again = app
        .post_complete_session(
            login.user_id.clone(),
            stripe_session_id_again.clone(),
            login.token.clone(),
        )
        .await;
    assert_eq!(&200, &complete_session_response_again.status().as_u16());

    //ASSERT WE HAVE 1 SUBSCRIPTION IN THE LIST!
    let subscription_list_response = app
        .get_subscriptions_by_subscriber_id(subscriber.id.to_string().clone(), login.token.clone())
        .await;
    assert_eq!(&200, &subscription_list_response.status().as_u16());
    let subscription_list_response_body = subscription_list_response.text().await.unwrap();
    let subscription_list: Vec<OverTheWireSubscription> =
        serde_json::from_str(subscription_list_response_body.as_str()).unwrap();
    assert_eq!(2, subscription_list.len());
}
