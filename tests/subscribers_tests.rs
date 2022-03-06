use reqwest::Client;
use uuid::Uuid;

use newsletter_signup_service::domain::new_subscriber::OverTheWireSubscriber;

use crate::util::{spawn_app, store_subscriber};

pub mod util;

#[tokio::test]
async fn subscribers_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = Client::new();

    let body = format!(
        "id={}&last_name=le%20guin&first_name=ursila&email_address=ursula_le_guin%40gmail.com",
        Uuid::new_v4().to_string()
    );
    let response = client
        .post(&format!("{}/subscribers", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email_address, first_name, last_name, id FROM subscribers")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email_address, "ursula_le_guin@gmail.com");
    assert_eq!(saved.last_name, "le guin");
}

#[tokio::test]
async fn given_a_stored_subscriber_i_can_get_it_by_email() {
    let app = spawn_app().await;
    let client: Client = Client::new();

    let subscriber = store_subscriber(app.clone()).await;

    let response = client
        .get(&format!(
            "{}/subscribers?email={}",
            app.address, subscriber.email_address
        ))
        .send()
        .await
        .expect("Got a subscriber back");
    assert!(response.status().is_success());

    let response_body = response.text().await.unwrap();
    let saved_subscriber: OverTheWireSubscriber =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(saved_subscriber.email_address, subscriber.email_address);
}

#[tokio::test]
async fn given_a_stored_subscriber_i_can_get_it_by_id() {
    let app = spawn_app().await;
    let client: Client = Client::new();

    let subscriber = store_subscriber(app.clone()).await;

    let response = client
        .get(&format!("{}/subscribers/{}", app.address, subscriber.id))
        .send()
        .await
        .expect("Got a subscriber back");
    assert!(response.status().is_success());

    let response_body = response.text().await.unwrap();
    let saved_subscriber: OverTheWireSubscriber =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(saved_subscriber.id, subscriber.id);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("first_name=ursila&last_name=le%20guin", "missing the email"),
        (
            "first_name=ursila&last_name=%20&email=ursula_le_guin%40gmail.com",
            "missing the name",
        ),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscribers", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
