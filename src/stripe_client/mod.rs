pub mod stripe_models;

use crate::stripe_client::stripe_models::{
    StripeBillingPortalSession, StripeCheckoutSession, StripeCustomer, StripePriceList,
    StripeProductPrice, StripeSessionObject,
};
use reqwest::{Client, Error};
use secrecy::{ExposeSecret, Secret};
use tracing::Level;
use urlencoding::encode;

pub const STRIPE_SESSIONS_BASE_PATH: &str = "/v1/checkout/sessions";
pub const STRIPE_SUBSCRIPTIONS_BASE_PATH: &str = "/v1/subscriptions/";
pub const STRIPE_BILLING_PORTAL_BASE_PATH: &str = "/v1/billing_portal/sessions";
pub const STRIPE_CUSTOMERS_BASE_PATH: &str = "/v1/customers";
pub const STRIPE_PRICES_BASE_PATH: &str = "/v1/prices";

#[derive(Clone, Debug)]
pub struct StripeClient {
    http_client: Client,
    base_url: String,
    api_secret_key: Secret<String>,
}

impl StripeClient {
    pub fn new(
        base_url: String,
        api_secret_key: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(timeout)
                .connection_verbose(true)
                .build()
                .unwrap(),
            base_url,
            api_secret_key,
        }
    }

    #[tracing::instrument(
    name = "Get Stripe Session By ID",
    skip(stripe_session_id),
    fields(
    stripe_session_id = % stripe_session_id,
    )
    )]
    pub async fn get_stripe_session(
        &self,
        stripe_session_id: String,
    ) -> Result<StripeSessionObject, Error> {
        let address = format!(
            "{}{}/{}",
            &self.base_url, STRIPE_SESSIONS_BASE_PATH, stripe_session_id
        );

        let response = self
            .http_client
            .get(address)
            .basic_auth(
                self.api_secret_key.expose_secret().to_string(),
                Option::Some(String::new()),
            )
            .send()
            .await?
            .error_for_status();

        return match response {
            Ok(response) => {
                let response_body = response.text().await.unwrap();
                tracing::event!(Level::INFO, "Got the following back!! {:?}", &response_body);
                let stripe_session: StripeSessionObject =
                    serde_json::from_str(response_body.as_str()).unwrap();
                return Ok(stripe_session);
            }
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(err)
            }
        };
    }

    #[tracing::instrument(
    name = "Cancel Stripe Subscription",
    skip(subscription_id),
    fields(
    subscription_id = % subscription_id,
    )
    )]
    pub async fn cancel_stripe_subscription(&self, subscription_id: String) -> Result<(), Error> {
        let address = format!(
            "{}{}{}",
            &self.base_url, STRIPE_SUBSCRIPTIONS_BASE_PATH, subscription_id
        );

        let response = reqwest::Client::new()
            .delete(address)
            .basic_auth(
                self.api_secret_key.expose_secret().to_string(),
                Option::Some(String::new()),
            )
            .send()
            .await?
            .error_for_status();

        match response {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(err)
            }
        }
    }

    #[tracing::instrument(
    name = "Create a billing portal session",
    skip(stripe_customer_id, return_url),
    fields(
    stripe_customer_id = % stripe_customer_id,
    )
    )]
    pub async fn create_billing_portal_session(
        &self,
        stripe_customer_id: String,
        return_url: String,
    ) -> Result<StripeBillingPortalSession, Error> {
        let address = format!(
            "{}{}?customer={}&return_url={}",
            &self.base_url, STRIPE_BILLING_PORTAL_BASE_PATH, stripe_customer_id, return_url
        );
        let response = self
            .http_client
            .post(address)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(
                self.api_secret_key.expose_secret().to_string(),
                Option::Some(String::new()),
            )
            .send()
            .await?
            .error_for_status();

        return match response {
            Ok(response) => {
                let response_body = response.text().await.unwrap();
                tracing::event!(Level::INFO, "Got the following back!! {:?}", &response_body);
                let stripe_session: StripeBillingPortalSession =
                    serde_json::from_str(response_body.as_str()).unwrap();
                Ok(stripe_session)
            }
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(err)
            }
        };
    }

    #[tracing::instrument(
    name = "Create a stripe customer",
    skip(email),
    fields(
    email = % email,
    )
    )]
    pub async fn create_stripe_customer(&self, email: String) -> Result<StripeCustomer, Error> {
        let address = format!(
            "{}{}?email={}",
            &self.base_url,
            STRIPE_CUSTOMERS_BASE_PATH,
            encode(email.as_str())
        );

        let create_customer_response = self
            .http_client
            .post(address)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(
                self.api_secret_key.expose_secret().to_string(),
                Option::Some(String::new()),
            )
            .send()
            .await?
            .error_for_status();

        return match create_customer_response {
            Ok(response) => {
                let response_body = response.text().await.unwrap();
                tracing::event!(Level::INFO, "Got the following back!! {:?}", &response_body);
                let stripe_customer: StripeCustomer =
                    serde_json::from_str(response_body.as_str()).unwrap();
                Ok(stripe_customer)
            }
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(err)
            }
        };
    }

    #[tracing::instrument(name = "Get Stripe Price By Lookup Key")]
    pub async fn get_stripe_price_by_lookup_key(
        &self,
        lookup_keys: Vec<String>,
    ) -> Result<StripePriceList, Error> {
        let mut keys_param: String = String::new();
        for key in lookup_keys.iter() {
            keys_param.push_str(format!("{}={}&", encode("lookup_keys[]"), key).as_str())
        }

        let address = format!(
            "{}{}?{}",
            &self.base_url, STRIPE_PRICES_BASE_PATH, keys_param,
        );

        let get_prices_response = self
            .http_client
            .get(address)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(
                self.api_secret_key.expose_secret().to_string(),
                Option::Some(String::new()),
            )
            .send()
            .await?
            .error_for_status();

        return match get_prices_response {
            Ok(response) => {
                let response_body = response.text().await.unwrap();
                tracing::event!(Level::INFO, "Got the following back!! {:?}", &response_body);
                let stripe_price_list: StripePriceList =
                    serde_json::from_str(response_body.as_str()).unwrap();
                Ok(stripe_price_list)
            }
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(err)
            }
        };
    }

    #[tracing::instrument(name = "Get Stripe Price By ID")]
    pub async fn get_stripe_price_by_id(&self, id: String) -> Result<StripeProductPrice, Error> {
        let address = format!("{}{}/{}", &self.base_url, STRIPE_PRICES_BASE_PATH, id,);

        let get_prices_response = self
            .http_client
            .get(address)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(
                self.api_secret_key.expose_secret().to_string(),
                Option::Some(String::new()),
            )
            .send()
            .await?
            .error_for_status();

        return match get_prices_response {
            Ok(response) => {
                let response_body = response.text().await.unwrap();
                tracing::event!(Level::INFO, "Got the following back!! {:?}", &response_body);
                let stripe_price: StripeProductPrice =
                    serde_json::from_str(response_body.as_str()).unwrap();
                Ok(stripe_price)
            }
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(err)
            }
        };
    }

    #[tracing::instrument(
    name = "Create a Stripe checkout session",
    skip(price_id, quantity, stripe_customer_id, success_url, cancel_url),
    fields(
    stripe_customer_id = % stripe_customer_id,
    )
    )]
    pub async fn create_stripe_checkout_session(
        &self,
        price_id: String,
        quantity: u16,
        stripe_customer_id: String,
        success_url: String,
        cancel_url: String,
        mode: String,
    ) -> Result<StripeCheckoutSession, Error> {
        let address = format!(
            "{}{}?success_url={}&cancel_url={}&{}={}&{}={}&mode={}&customer={}",
            &self.base_url,
            STRIPE_SESSIONS_BASE_PATH,
            success_url,
            cancel_url,
            "line_items[0][price]",
            price_id,
            "line_items[0][quantity]",
            quantity,
            mode,
            stripe_customer_id,
        );

        let create_checkout_session_response = self
            .http_client
            .post(address)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(
                self.api_secret_key.expose_secret().to_string(),
                Option::Some(String::new()),
            )
            .send()
            .await?
            .error_for_status();

        return match create_checkout_session_response {
            Ok(response) => {
                let response_body = response.text().await.unwrap();
                tracing::event!(Level::INFO, "Got the following back!! {:?}", &response_body);
                let stripe_checkout_session: StripeCheckoutSession =
                    serde_json::from_str(response_body.as_str()).unwrap();
                Ok(stripe_checkout_session)
            }
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(err)
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{Fake, Faker};
    use secrecy::Secret;

    use crate::stripe_client::stripe_models::{
        StripeBillingPortalSession, StripeCheckoutSession, StripeCustomer, StripePriceList,
        StripeProductPrice, StripeSessionObject,
    };
    use uuid::Uuid;
    use wiremock::matchers::{header_exists, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::stripe_client::{
        StripeClient, STRIPE_BILLING_PORTAL_BASE_PATH, STRIPE_CUSTOMERS_BASE_PATH,
        STRIPE_PRICES_BASE_PATH, STRIPE_SESSIONS_BASE_PATH, STRIPE_SUBSCRIPTIONS_BASE_PATH,
    };

    fn stripe_client(base_url: String) -> StripeClient {
        StripeClient::new(
            base_url,
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn get_stripe_session_works() {
        // Arrange
        let session_id = Uuid::new_v4().to_string();
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());
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

        // Act
        let outcome = stripe_client.get_stripe_session(session_id).await;
        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn get_stripe_session_returns_error_when_it_is_an_error() {
        // Arrange
        let session_id = Uuid::new_v4().to_string();
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        Mock::given(header_exists("Authorization"))
            .and(path(format!(
                "{}/{}",
                STRIPE_SESSIONS_BASE_PATH, &session_id
            )))
            .and(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client.get_stripe_session(session_id).await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn cancel_stripe_subscription_works() {
        // Arrange
        let subscription_id = Uuid::new_v4().to_string();
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());
        let response = ResponseTemplate::new(200);

        Mock::given(header_exists("Authorization"))
            .and(path(format!(
                "{}{}",
                STRIPE_SUBSCRIPTIONS_BASE_PATH, &subscription_id
            )))
            .and(method("DELETE"))
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client
            .cancel_stripe_subscription(subscription_id)
            .await;
        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn cancel_stripe_subscription_returns_error_when_it_is_an_error() {
        // Arrange
        let subscription_id = Uuid::new_v4().to_string();
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        Mock::given(header_exists("Authorization"))
            .and(path(format!(
                "{}{}",
                STRIPE_SUBSCRIPTIONS_BASE_PATH, &subscription_id
            )))
            .and(method("DELETE"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client
            .cancel_stripe_subscription(subscription_id)
            .await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn create_billing_portal_session_works() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        let customer_id = Uuid::new_v4().to_string();
        let return_url = Uuid::new_v4().to_string();

        let billing_portal = StripeBillingPortalSession {
            id: Uuid::new_v4().to_string(),
            object: "something".to_string(),
            configuration: "something".to_string(),
            created: 12341234,
            customer: customer_id.clone(),
            livemode: false,
            locale: None,
            on_behalf_of: None,
            return_url: return_url.clone(),
            url: Uuid::new_v4().to_string(),
        };

        let response = ResponseTemplate::new(200).set_body_json(serde_json::json!(billing_portal));

        Mock::given(header_exists("Authorization"))
            .and(path(STRIPE_BILLING_PORTAL_BASE_PATH))
            .and(query_param("customer", &customer_id))
            .and(query_param("return_url", &return_url))
            .and(method("POST"))
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client
            .create_billing_portal_session(customer_id, return_url)
            .await;
        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn create_billing_portal_session_returns_error_when_it_is_an_error() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        let customer_id = Uuid::new_v4().to_string();
        let return_url = Uuid::new_v4().to_string();

        Mock::given(header_exists("Authorization"))
            .and(path(STRIPE_BILLING_PORTAL_BASE_PATH))
            .and(query_param("customer", &customer_id))
            .and(query_param("return_url", &return_url))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client
            .create_billing_portal_session(customer_id, return_url)
            .await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn create_stripe_customer_works() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        let customer_id = Uuid::new_v4().to_string();
        let customer_email = Uuid::new_v4().to_string();

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

        // Act
        let outcome = stripe_client.create_stripe_customer(customer_email).await;
        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn create_stripe_customer_returns_error_when_it_is_an_error() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());
        let customer_email = Uuid::new_v4().to_string();

        Mock::given(header_exists("Authorization"))
            .and(path(STRIPE_CUSTOMERS_BASE_PATH))
            .and(query_param("email", &customer_email))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client.create_stripe_customer(customer_email).await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn get_stripe_price_by_lookup_key_works() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        let stripe_lookup_key = Some(Uuid::new_v4().to_string());

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
            .and(query_param(
                "lookup_keys[]",
                stripe_lookup_key.clone().unwrap(),
            ))
            .and(method("GET"))
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client
            .get_stripe_price_by_lookup_key(vec![stripe_lookup_key.unwrap()])
            .await;
        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn get_stripe_price_by_id_works() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        let stripe_lookup_key = Some(Uuid::new_v4().to_string());

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

        let response = ResponseTemplate::new(200).set_body_json(serde_json::json!(price));

        Mock::given(header_exists("Authorization"))
            .and(path(format!(
                "{}/{}",
                STRIPE_PRICES_BASE_PATH,
                price.id.clone()
            )))
            .and(method("GET"))
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client.get_stripe_price_by_id(price.id.clone()).await;
        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn get_stripe_price_by_id_returns_error() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        let id = Uuid::new_v4().to_string();
        let response = ResponseTemplate::new(500);

        Mock::given(header_exists("Authorization"))
            .and(path(format!("{}/{}", STRIPE_PRICES_BASE_PATH, id.clone())))
            .and(method("GET"))
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client.get_stripe_price_by_id(id.clone()).await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn get_stripe_price_by_lookup_key_returns_error_when_it_is_an_error() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());
        let stripe_lookup_key = Uuid::new_v4().to_string();

        Mock::given(header_exists("Authorization"))
            .and(path(STRIPE_PRICES_BASE_PATH))
            .and(query_param("lookup_keys[]", stripe_lookup_key.clone()))
            .and(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client
            .get_stripe_price_by_lookup_key(vec![stripe_lookup_key])
            .await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn create_stripe_checkout_session_works() {
        // Arrange
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());

        let success_url = Uuid::new_v4().to_string();
        let cancel_url = Uuid::new_v4().to_string();
        let price_id = Uuid::new_v4().to_string();
        let quantity = 1;
        let mode = "something".to_string();
        let stripe_customer_id = Uuid::new_v4().to_string();

        let checkout_session = StripeCheckoutSession {
            id: Uuid::new_v4().to_string(),
            object: "checkout.session".to_string(),
            cancel_url: cancel_url.clone(),
            url: Uuid::new_v4().to_string(),
        };

        let response =
            ResponseTemplate::new(200).set_body_json(serde_json::json!(checkout_session));

        Mock::given(header_exists("Authorization"))
            .and(path(STRIPE_SESSIONS_BASE_PATH))
            .and(query_param("success_url", &success_url))
            .and(query_param("cancel_url", &cancel_url))
            .and(query_param("line_items[0][price]", &price_id))
            .and(query_param(
                "line_items[0][quantity]",
                &quantity.to_string(),
            ))
            .and(query_param("mode", &mode))
            .and(query_param("customer", &stripe_customer_id))
            .and(method("POST"))
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = stripe_client
            .create_stripe_checkout_session(
                price_id,
                quantity,
                stripe_customer_id,
                success_url,
                cancel_url,
                mode,
            )
            .await;
        // Assert
        assert_ok!(outcome);
    }
}
