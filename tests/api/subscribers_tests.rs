use uuid::Uuid;

use newsletter_signup_service::domain::new_subscriber::{
    OverTheWireCreateSubscriber, OverTheWireSubscriber,
};

use crate::helper::{generate_over_the_wire_subscriber, spawn_app, store_subscriber};

#[tokio::test]
async fn subscribers_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let subscriber = generate_over_the_wire_subscriber();
    let response = app.post_subscriber(subscriber.to_json()).await;

    assert_eq!(200, response.status().as_u16());

    let find_response = app
        .get_subscriber_by_email(subscriber.email_address.clone())
        .await;
    assert_eq!(200, find_response.status().as_u16());

    let saved: OverTheWireSubscriber = app
        .from_response_to_over_the_wire_subscriber(find_response)
        .await;
    assert_eq!(saved.email_address, subscriber.email_address);
    assert_eq!(saved.last_name, subscriber.last_name);
}

#[tokio::test]
async fn given_a_stored_subscriber_i_can_get_it_by_email() {
    let app = spawn_app().await;

    let subscriber = store_subscriber(app.clone(), Option::None).await;

    let response = app
        .get_subscriber_by_email(subscriber.email_address.clone())
        .await;
    assert!(response.status().is_success());

    let response_body = response.text().await.unwrap();
    let saved_subscriber: OverTheWireSubscriber =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(saved_subscriber.email_address, subscriber.email_address);
}

#[tokio::test]
async fn incorrect_email_returns_404() {
    let app = spawn_app().await;

    store_subscriber(app.clone(), Option::None).await;

    let response = app
        .get_subscriber_by_email(Uuid::new_v4().to_string())
        .await;
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn incorrect_id_returns_404() {
    let app = spawn_app().await;

    store_subscriber(app.clone(), Option::None).await;

    let response = app.get_subscriber_by_id(Uuid::new_v4().to_string()).await;
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn given_a_stored_subscriber_i_can_get_it_by_id() {
    let app = spawn_app().await;

    let subscriber = store_subscriber(app.clone(), Option::None).await;

    let response = app.get_subscriber_by_id(subscriber.id.clone()).await;
    assert!(response.status().is_success());

    let saved_subscriber = app
        .from_response_to_over_the_wire_subscriber(response)
        .await;

    assert_eq!(saved_subscriber.id, subscriber.id);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            OverTheWireCreateSubscriber {
                last_name: Uuid::new_v4().to_string(),
                first_name: Uuid::new_v4().to_string(),
                email_address: String::from(""),
            },
            "missing the email",
        ),
        (
            OverTheWireCreateSubscriber {
                last_name: String::from(""),
                first_name: Uuid::new_v4().to_string(),
                email_address: Uuid::new_v4().to_string(),
            },
            "missing the name",
        ),
        (
            OverTheWireCreateSubscriber {
                last_name: String::from(""),
                first_name: Uuid::new_v4().to_string(),
                email_address: String::from(""),
            },
            "missing both name and email",
        ),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriber(invalid_body.to_json()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;

    // Sabotage the database
    sqlx::query!("ALTER TABLE subscribers DROP COLUMN last_name;",)
        .execute(&app.db_pool)
        .await
        .unwrap();
    // Act
    let response = app
        .post_subscriber(generate_over_the_wire_subscriber().to_json())
        .await;
    // Assert
    assert_eq!(response.status().as_u16(), 500);
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = spawn_app().await;
    let response = reqwest::Client::new()
        .post(&format!("{}/subscribers", app.address))
        .header("Content-Type", "application/json")
        .body(generate_over_the_wire_subscriber().to_json())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="subscribe""#,
        response.headers()["WWW-Authenticate"]
    );
}
