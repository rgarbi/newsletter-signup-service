pub mod stripe_models;

use crate::stripe_client::stripe_models::{
    StripeBillingPortalSession, StripeCustomer, StripeSessionObject,
};
use reqwest::{Client, Error};
use secrecy::{ExposeSecret, Secret};
use tracing::Level;

pub const STRIPE_SESSIONS_BASE_PATH: &str = "/v1/checkout/sessions/";
pub const STRIPE_SUBSCRIPTIONS_BASE_PATH: &str = "/v1/subscriptions/";
pub const STRIPE_BILLING_PORTAL_BASE_PATH: &str = "/v1/billing_portal/sessions";
pub const STRIPE_CUSTOMERS_BASE_PATH: &str = "/v1/customers";

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
        stripe_session_id = %stripe_session_id,
        )
    )]
    pub async fn get_stripe_session(
        &self,
        stripe_session_id: String,
    ) -> Result<StripeSessionObject, Error> {
        let address = format!(
            "{}{}{}",
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
            subscription_id = %subscription_id,
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

        return match response {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(err)
            }
        };
    }

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

    pub async fn create_stripe_customer(&self, email: String) -> Result<StripeCustomer, Error> {
        let address = format!(
            "{}{}?email={}",
            &self.base_url, STRIPE_CUSTOMERS_BASE_PATH, email
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
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::{Fake, Faker};
    use secrecy::Secret;

    use crate::stripe_client::stripe_models::{
        StripeBillingPortalSession, StripeCustomer, StripeSessionObject,
    };
    use uuid::Uuid;
    use wiremock::matchers::{header_exists, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::stripe_client::{
        StripeClient, STRIPE_BILLING_PORTAL_BASE_PATH, STRIPE_CUSTOMERS_BASE_PATH,
        STRIPE_SESSIONS_BASE_PATH, STRIPE_SUBSCRIPTIONS_BASE_PATH,
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
                "{}{}",
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
                "{}{}",
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
}
