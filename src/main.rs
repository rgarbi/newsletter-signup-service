use std::net::TcpListener;

use newsletter_signup_service::configuration::get_configuration;
use newsletter_signup_service::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read the configuration.");
    let address = format!("0.0.0.0:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}
