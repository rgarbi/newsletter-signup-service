use std::net::TcpListener;

use newsletter_signup_service::startup::run;
use reqwest::Client;

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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_orm_data() {
    let app_address = spawn_app();
    let client = Client::new();

    let body = "last_name=le%20guin&first_name=ursila&email_address=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscribers", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("first_name=ursila&last_name=le%20guin", "missing the email"),
        (
            "first_name=ursila&email=ursula_le_guin%40gmail.com",
            "missing the name",
        ),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscribers", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

fn spawn_app() -> String {
    let listener: TcpListener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to start the server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
