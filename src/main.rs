use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;

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
    let connection = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
