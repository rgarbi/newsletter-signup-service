use reqwest::Client;
use secrecy::Secret;

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
}
