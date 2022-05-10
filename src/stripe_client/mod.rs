pub mod stripe_models;

use crate::stripe_client::stripe_models::StripeSessionObject;
use reqwest::{Client, Error};
use secrecy::{ExposeSecret, Secret};
use tracing::Level;

pub const STRIPE_SESSIONS_BASE_PATH: &str = "/v1/checkout/sessions/";
pub const STRIPE_SUBSCRIPTIONS_BASE_PATH: &str = "/v1/subscriptions/";

#[derive(Clone, Debug)]
pub struct StripeClient {
    http_client: Client,
    base_url: String,
    api_secret_key: Secret<String>,
    api_public_key: Secret<String>,
    webhook_secret: Secret<String>,
}

impl StripeClient {
    pub fn new(
        base_url: String,
        api_secret_key: Secret<String>,
        api_public_key: Secret<String>,
        webhook_secret: Secret<String>,
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
            api_public_key,
            webhook_secret,
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
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::{Fake, Faker};
    use secrecy::Secret;

    use crate::stripe_client::stripe_models::StripeSessionObject;
    use uuid::Uuid;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use crate::stripe_client::{StripeClient, STRIPE_SESSIONS_BASE_PATH};

    fn stripe_client(base_url: String) -> StripeClient {
        StripeClient::new(
            base_url,
            Secret::new(Faker.fake()),
            Secret::new(Faker.fake()),
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
    async fn cancel_stripe_subscription_works() {
        // Arrange
        let subscription_id = Uuid::new_v4().to_string();
        let mock_server = MockServer::start().await;
        let stripe_client = stripe_client(mock_server.uri());
        let response = ResponseTemplate::new(200);

        Mock::given(header_exists("Authorization"))
            .and(path(format!("/v1/checkout/sessions/{}", &subscription_id)))
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
}
