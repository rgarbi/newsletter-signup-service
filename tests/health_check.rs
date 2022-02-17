use reqwest::Client;

use newsletter_signup_service::run;

#[tokio::test]
async fn health_check_works() {
    spawn_app();
    let client: Client = Client::new();

    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let server = run().expect("Failed to start the server");
    let _ = tokio::spawn(server);
}
