use newsletter_signup_service::configuration::{current_environment, get_configuration, Environment};
use newsletter_signup_service::startup::Application;
use newsletter_signup_service::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let default_filter = match current_environment() {
        // Local: app info + SQL statements (sqlx uses target `sqlx::query` at TRACE).
        Environment::Local => "info,sqlx::query=trace",
        Environment::Production => "error",
    };
    let subscriber = get_subscriber(
        "newsletter-signup-service".into(),
        default_filter.into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
