use once_cell::sync::Lazy;
use reqwest::Client;

use newsletter_signup_service::configuration::get_configuration;

use crate::util::{spawn_app, TRACING};

pub mod util;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client: Client = Client::new();

    let response = client
        .get(&format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn config_override_in_prod_works() {
    Lazy::force(&TRACING);

    std::env::set_var("APP__DATABASE__HOST", "localhost");

    let configuration = get_configuration().expect("Failed to read configuration");
    assert_eq!("localhost", configuration.database.host);

    std::env::remove_var("APP__DATABASE__HOST");

    let configuration = get_configuration().expect("Failed to read configuration");
    assert_eq!("127.0.0.1", configuration.database.host);
}
