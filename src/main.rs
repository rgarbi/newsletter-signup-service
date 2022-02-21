use std::net::TcpListener;

use newsletter_signup_service::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run(TcpListener::bind("0.0.0.0:8000").expect("Failed to bind to our port!"))?.await
}
