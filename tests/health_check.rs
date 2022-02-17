use reqwest::Client;
use std::net::TcpListener;

use newsletter_signup_service::run;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client: Client = Client::new();

    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener: TcpListener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to start the server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
