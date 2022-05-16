use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{header_exists, method, path, query_param};
use newsletter_signup_service::auth::token::LoginResponse;
use newsletter_signup_service::domain::checkout_models::{CreateCheckoutSession, CreateStripeSessionRedirect};
use newsletter_signup_service::domain::subscriber_models::OverTheWireSubscriber;
use newsletter_signup_service::stripe_client::{STRIPE_CUSTOMERS_BASE_PATH, STRIPE_PRICES_BASE_PATH, STRIPE_SESSIONS_BASE_PATH};
use newsletter_signup_service::stripe_client::stripe_models::{StripeCheckoutSession, StripeCustomer, StripePriceList, StripeProductPrice, StripeSessionObject};
use crate::helper::{generate_over_the_wire_create_subscription, generate_signup, spawn_app};



/*
    1. Create a user account
    2. Sign up for a subscription.
    3. Complete Payment.
*/
#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    //SIGN UP
    let sign_up = generate_signup();
    let sign_up_response= app.user_signup(sign_up.to_json()).await;
    assert_eq!(&200, &sign_up_response.status().as_u16());
    let sign_up_response_body = sign_up_response.text().await.unwrap();
    let login: LoginResponse = serde_json::from_str(sign_up_response_body.as_str()).unwrap();

    //GET SUBSCRIBER BY USER ID
    let get_subscriber_by_user_id_response = app.get_subscriber_by_user_id(login.user_id.clone(), login.token.clone()).await;
    assert_eq!(&200, &get_subscriber_by_user_id_response.status().as_u16());
    let subscriber_response_body = get_subscriber_by_user_id_response.text().await.unwrap();
    let subscriber: OverTheWireSubscriber = serde_json::from_str(subscriber_response_body.as_str()).unwrap();

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
}


async fn mock_stripe_create_customer(mock_server: &MockServer, customer_email: String) {
    let stripe_customer = StripeCustomer {
        id: Uuid::new_v4().to_string(),
        object: "something".to_string(),
        created: 12341234,
        description: None,
        email: Some(customer_email.clone()),
    };

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!(stripe_customer));

    Mock::given(header_exists("Authorization"))
        .and(path(STRIPE_CUSTOMERS_BASE_PATH))
        .and(query_param("email", &customer_email))
        .and(method("POST"))
        .respond_with(response)
        .expect(1)
        .mount(&mock_server)
        .await;
}

async fn mock_stripe_price_lookup(mock_server: &MockServer, stripe_lookup_key: String) {
    let price = StripeProductPrice {
        id: Uuid::new_v4().to_string(),
        object: "price".to_string(),
        active: true,
        billing_scheme: "".to_string(),
        created: 12341234,
        currency: "".to_string(),
        product: "".to_string(),
        unit_amount: 500,
        lookup_key: stripe_lookup_key.clone(),
    };

    let price_list: Vec<StripeProductPrice> = vec![price];

    let stripe_price_search_list = StripePriceList {
        object: "list".to_string(),
        url: "/v1/prices".to_string(),
        has_more: false,
        data: price_list,
    };

    let response =
        ResponseTemplate::new(200).set_body_json(serde_json::json!(stripe_price_search_list));

    Mock::given(header_exists("Authorization"))
        .and(path(STRIPE_PRICES_BASE_PATH))
        .and(query_param("lookup_keys[]", stripe_lookup_key.clone()))
        .and(method("GET"))
        .respond_with(response)
        .expect(1)
        .mount(&mock_server)
        .await;
}

async fn mock_create_checkout_session(mock_server: &MockServer, stripe_session_id: String) {
    let checkout_session = StripeCheckoutSession {
        id: stripe_session_id,
        object: "checkout.session".to_string(),
        cancel_url: Uuid::new_v4().to_string(),
        url: Uuid::new_v4().to_string(),
    };

    let response =
        ResponseTemplate::new(200).set_body_json(serde_json::json!(checkout_session));

    Mock::given(header_exists("Authorization"))
        .and(path(STRIPE_SESSIONS_BASE_PATH))
        .and(method("POST"))
        .respond_with(response)
        .expect(1)
        .mount(&mock_server)
        .await;
}

async fn mock_get_stripe_session(mock_server: &MockServer, session_id: String) {
    let stripe_session = StripeSessionObject {
        id: session_id.clone(),
        object: "something".to_string(),
        amount_subtotal: 500,
        amount_total: 500,
        client_reference_id: None,
        customer: Uuid::new_v4().to_string(),
        subscription: Some(Uuid::new_v4().to_string()),
    };

    let response = ResponseTemplate::new(200)
        .set_body_json(serde_json::json!(stripe_session))
        .append_header("Content-Type", "application/json");

    Mock::given(header_exists("Authorization"))
        .and(path(format!(
            "{}/{}",
            STRIPE_SESSIONS_BASE_PATH, &session_id
        )))
        .and(method("GET"))
        .respond_with(response)
        .expect(1)
        .mount(&mock_server)
        .await;
}
