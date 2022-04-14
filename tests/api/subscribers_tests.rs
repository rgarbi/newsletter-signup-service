use uuid::Uuid;

use newsletter_signup_service::auth::token::generate_token;
use newsletter_signup_service::domain::subscriber_models::{
    OverTheWireCreateSubscriber, OverTheWireSubscriber,
};

use crate::helper::{generate_over_the_wire_subscriber, spawn_app};

#[tokio::test]
async fn subscribers_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let subscriber = generate_over_the_wire_subscriber();
    let response = app
        .post_subscriber(
            subscriber.to_json(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;

    assert_eq!(200, response.status().as_u16());

    let find_response = app
        .get_subscriber_by_email(
            subscriber.email_address.clone(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(200, find_response.status().as_u16());

    let saved: OverTheWireSubscriber = app
        .from_response_to_over_the_wire_subscriber(find_response)
        .await;
    assert_eq!(saved.email_address, subscriber.email_address);
    assert_eq!(saved.name, subscriber.name);
}

#[tokio::test]
async fn given_a_stored_subscriber_i_can_get_it_by_email() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_email(
            subscriber.email_address.clone(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert!(response.status().is_success());

    let response_body = response.text().await.unwrap();
    let saved_subscriber: OverTheWireSubscriber =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(saved_subscriber.email_address, subscriber.email_address);
}

#[tokio::test]
async fn given_a_stored_subscriber_i_can_get_it_by_user_id() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_user_id(
            subscriber.user_id.clone(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert!(response.status().is_success());

    let response_body = response.text().await.unwrap();
    let saved_subscriber: OverTheWireSubscriber =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(saved_subscriber.user_id, subscriber.user_id);
}

#[tokio::test]
async fn given_a_stored_subscriber_i_can_get_it_by_user_id_and_email() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_user_id_and_email(
            subscriber.user_id.clone(),
            subscriber.email_address.clone(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert!(response.status().is_success());

    let response_body = response.text().await.unwrap();
    let saved_subscriber: OverTheWireSubscriber =
        serde_json::from_str(response_body.as_str()).unwrap();

    assert_eq!(saved_subscriber.user_id, subscriber.user_id);
}

#[tokio::test]
async fn empty_user_id_and_email_gives_404() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_user_id_and_email(
            String::new(),
            String::new(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn bad_user_id_and_email_gives_404() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_user_id_and_email(
            Uuid::new_v4().to_string(),
            Uuid::new_v4().to_string(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn bad_user_id_gives_404() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_user_id_and_email(
            Uuid::new_v4().to_string(),
            subscriber.email_address.clone(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn bad_email_gives_404() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_user_id_and_email(
            subscriber.user_id.clone(),
            Uuid::new_v4().to_string(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn bad_token_gives_401() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_user_id_and_email(
            subscriber.user_id.clone(),
            subscriber.email_address.clone(),
            generate_token(Uuid::new_v4().to_string()),
        )
        .await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn incorrect_email_returns_404() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_email(
            Uuid::new_v4().to_string(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn incorrect_id_returns_404() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_id(
            Uuid::new_v4().to_string(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn given_a_stored_subscriber_i_can_get_it_by_id() {
    let app = spawn_app().await;

    let subscriber = app.store_subscriber(Option::None).await;

    let response = app
        .get_subscriber_by_id(
            subscriber.id.clone(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
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
                name: Uuid::new_v4().to_string(),
                email_address: String::from(""),
                user_id: Uuid::new_v4().to_string(),
            },
            "missing the email",
        ),
        (
            OverTheWireCreateSubscriber {
                name: String::from(""),
                email_address: Uuid::new_v4().to_string(),
                user_id: Uuid::new_v4().to_string(),
            },
            "missing the name",
        ),
        (
            OverTheWireCreateSubscriber {
                name: String::from(""),
                email_address: String::from(""),
                user_id: Uuid::new_v4().to_string(),
            },
            "missing both name and email",
        ),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = app
            .post_subscriber(
                invalid_body.to_json(),
                generate_token(invalid_body.user_id.clone()),
            )
            .await;

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
    sqlx::query!("ALTER TABLE subscribers DROP COLUMN name;",)
        .execute(&app.db_pool)
        .await
        .unwrap();
    // Act
    let subscriber = generate_over_the_wire_subscriber();
    let response = app
        .post_subscriber(
            subscriber.to_json(),
            generate_token(subscriber.user_id.clone()),
        )
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
}
