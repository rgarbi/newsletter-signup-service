use newsletter_signup_service::configuration::get_configuration;
use newsletter_signup_service::startup::Application;
use newsletter_signup_service::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber(
        "newsletter-signup-service".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    println!("Database host: {}", configuration.database.host);
    println!("Database username: {}", configuration.database.username);
    println!("Email base_url: {}", configuration.email_client.base_url);
    println!(
        "AUTH KEY: {}",
        configuration.auth_config.signing_key.expose_secret()
    );
    println!("Email base_url: {}", configuration.email_client.base_url);
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
