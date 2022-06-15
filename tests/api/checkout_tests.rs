use claim::assert_ok;
use newsletter_signup_service::auth::token::{generate_token, LoginResponse};
use newsletter_signup_service::configuration::get_configuration;
use newsletter_signup_service::db::checkout_session_db_broker::insert_checkout_session;
use newsletter_signup_service::domain::checkout_models::CreateCheckoutSession;
use newsletter_signup_service::domain::subscriber_models::OverTheWireSubscriber;
use newsletter_signup_service::domain::subscription_models::{
    OverTheWireSubscription, SubscriptionType,
};
use newsletter_signup_service::domain::user_models::UserGroup;
use uuid::Uuid;

use crate::helper::{
    generate_checkout_session, generate_new_subscription,
    generate_over_the_wire_create_subscription, generate_signup, mock_create_checkout_session,
    mock_create_stripe_billing_portal_session, mock_get_stripe_session,
    mock_stripe_create_customer, mock_stripe_create_customer_returns_a_500,
    mock_stripe_create_session_returns_a_500, mock_stripe_price_lookup,
    mock_stripe_price_lookup_returns_a_500, spawn_app,
};

#[tokio::test]
async fn create_checkout_session_not_authorized() {
    let app = spawn_app().await;

    let subscription = generate_over_the_wire_create_subscription(Uuid::new_v4().to_string(), None);
    let create_checkout_session = CreateCheckoutSession {
        price_lookup_key: Uuid::new_v4().to_string(),
        subscription,
    };

    let checkout_response = app
        .post_checkout(
            create_checkout_session.to_json(),
            Uuid::new_v4().to_string(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(401, checkout_response.status().as_u16());
}

#[tokio::test]
async fn create_checkout_session_subscriber_not_found() {
    let app = spawn_app().await;
    //SIGN UP
    let sign_up = generate_signup();
    let sign_up_response = app.user_signup(sign_up.to_json()).await;
    assert_eq!(&200, &sign_up_response.status().as_u16());
    let sign_up_response_body = sign_up_response.text().await.unwrap();
    let login: LoginResponse = serde_json::from_str(sign_up_response_body.as_str()).unwrap();

    let price_lookup_key = Uuid::new_v4().to_string();
    let subscription = generate_over_the_wire_create_subscription(
        Uuid::new_v4().to_string(),
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
    assert_eq!(400, checkout_response.status().as_u16());
}

#[tokio::test]
async fn create_checkout_session_stripe_customer_blows_up() {
    let app = spawn_app().await;

    //SIGN UP
    let login: LoginResponse = app.sign_up().await;

    //GET SUBSCRIBER BY USER ID
    let subscriber: OverTheWireSubscriber = app
        .get_subscriber_by_user_id(login.user_id.clone(), login.token.clone())
        .await;

    //SUBSCRIBE!
    mock_stripe_create_customer_returns_a_500(&app.stripe_server, subscriber.email_address.clone())
        .await;
    let subscription = generate_over_the_wire_create_subscription(
        subscriber.id.to_string().clone(),
        Some(SubscriptionType::Paper),
    );
    let create_checkout_session = CreateCheckoutSession {
        price_lookup_key: Uuid::new_v4().to_string(),
        subscription,
    };

    let checkout_response = app
        .post_checkout(
            create_checkout_session.to_json(),
            login.user_id.clone(),
            login.token.clone(),
        )
        .await;
    assert_eq!(500, checkout_response.status().as_u16());
}

#[tokio::test]
async fn create_checkout_session_cannot_find_prices() {
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
    mock_stripe_create_customer(&app.stripe_server, subscriber.email_address.clone()).await;
    mock_stripe_price_lookup_returns_a_500(&app.stripe_server, price_lookup_key.clone()).await;

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
    assert_eq!(500, checkout_response.status().as_u16());
}

#[tokio::test]
async fn create_checkout_session_fails() {
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
    mock_stripe_create_customer(&app.stripe_server, subscriber.email_address.clone()).await;
    mock_stripe_price_lookup(&app.stripe_server, price_lookup_key.clone()).await;
    mock_stripe_create_session_returns_a_500(&app.stripe_server).await;

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
    assert_eq!(500, checkout_response.status().as_u16());
}

#[tokio::test]
async fn store_create_checkout_result_fails() {
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

    let subscription = generate_over_the_wire_create_subscription(
        subscriber.id.to_string().clone(),
        Some(SubscriptionType::Paper),
    );
    let create_checkout_session = CreateCheckoutSession {
        price_lookup_key: price_lookup_key.clone(),
        subscription,
    };

    // Sabotage the database
    sqlx::query!("ALTER TABLE checkout_session DROP COLUMN subscription;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let checkout_response = app
        .post_checkout(
            create_checkout_session.to_json(),
            login.user_id.clone(),
            login.token.clone(),
        )
        .await;
    assert_eq!(500, checkout_response.status().as_u16());
}

#[tokio::test]
async fn complete_session_not_authorized() {
    let app = spawn_app().await;

    let user_id = Uuid::new_v4().to_string();
    let stripe_session_id = Uuid::new_v4().to_string();

    //store subscription
    let checkout_session = generate_checkout_session(Some(stripe_session_id.clone()));
    let result = insert_checkout_session(
        user_id.clone(),
        checkout_session.price_lookup_key,
        generate_new_subscription(Uuid::new_v4().to_string()),
        stripe_session_id.clone(),
        &app.db_pool,
    )
    .await;
    assert_ok!(result);

    //COMPLETE SUBSCRIPTION
    let complete_session_response = app
        .post_complete_session(
            user_id.clone(),
            stripe_session_id.clone(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(401, complete_session_response.status().as_u16());

    let someone_else = Uuid::new_v4().to_string();
    let complete_session_different_user_response = app
        .post_complete_session(
            someone_else.clone(),
            stripe_session_id.clone(),
            generate_token(someone_else, UserGroup::USER),
        )
        .await;
    assert_eq!(
        401,
        complete_session_different_user_response.status().as_u16()
    );
}

#[tokio::test]
async fn complete_session_not_found() {
    let app = spawn_app().await;

    let user_id = Uuid::new_v4().to_string();
    let stripe_session_id = Uuid::new_v4().to_string();

    //store subscription
    let checkout_session = generate_checkout_session(Some(stripe_session_id.clone()));
    let result = insert_checkout_session(
        user_id.clone(),
        checkout_session.price_lookup_key,
        generate_new_subscription(Uuid::new_v4().to_string()),
        stripe_session_id.clone(),
        &app.db_pool,
    )
    .await;
    assert_ok!(result);

    //COMPLETE SUBSCRIPTION
    let complete_session_response = app
        .post_complete_session(
            user_id.clone(),
            Uuid::new_v4().to_string(),
            generate_token(user_id, UserGroup::USER),
        )
        .await;
    assert_eq!(404, complete_session_response.status().as_u16());
}

#[tokio::test]
async fn create_stripe_portal_session_works() {
    let app = spawn_app().await;
    let config = get_configuration().unwrap().stripe_client;

    //SIGN UP
    let login: LoginResponse = app.sign_up().await;

    //GET SUBSCRIBER BY USER ID
    let mut subscriber: OverTheWireSubscriber = app
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

    //Refresh the subscriber. We should now have a stripe customer id.
    subscriber = app
        .get_subscriber_by_user_id(login.user_id.clone(), login.token.clone())
        .await;

    //ASSERT WE HAVE 1 SUBSCRIPTION IN THE LIST!
    let subscription_list_response = app
        .get_subscriptions_by_subscriber_id(subscriber.id.to_string().clone(), login.token.clone())
        .await;
    assert_eq!(&200, &subscription_list_response.status().as_u16());
    let subscription_list_response_body = subscription_list_response.text().await.unwrap();
    let subscription_list: Vec<OverTheWireSubscription> =
        serde_json::from_str(subscription_list_response_body.as_str()).unwrap();
    assert_eq!(1, subscription_list.len());

    //Create Portal session
    mock_create_stripe_billing_portal_session(
        &app.stripe_server,
        subscriber.stripe_customer_id.unwrap().to_string(),
    )
    .await;
    let create_portal_session_response = app
        .post_create_stripe_portal_session(login.user_id.clone(), login.token.clone())
        .await;
    assert_eq!(&200, &create_portal_session_response.status().as_u16());
}
