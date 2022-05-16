use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{header_exists, method, path, query_param};
use newsletter_signup_service::stripe_client::{STRIPE_CUSTOMERS_BASE_PATH, STRIPE_PRICES_BASE_PATH};
use newsletter_signup_service::stripe_client::stripe_models::{StripeCustomer, StripePriceList, StripeProductPrice};
use crate::helper::{spawn_app, TestApp};



/*
    1. Create a user account
    2. Sign up for a subscription.
    3. Complete Payment.
*/
#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let stored_subscription = store_subscription(subscriber.id.to_string(), &app).await;

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


async fn mock_stripe_create_customer(mock_server: MockServer, customer_email: String, customer_id: String) {
    let stripe_customer = StripeCustomer {
        id: customer_id,
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

async fn mock_stripe_price_lookup(mock_server: MockServer, stripe_lookup_key: String) {
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
