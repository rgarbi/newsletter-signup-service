use std::net::TcpListener;

use sqlx::PgPool;

use newsletter_signup_service::configuration::get_configuration;
use newsletter_signup_service::startup::run;
use newsletter_signup_service::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber(
        "newsletter-signup-service".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
