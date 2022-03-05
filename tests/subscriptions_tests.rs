use reqwest::Client;
use uuid::Uuid;

use crate::util::{spawn_app, store_subscriber};

pub mod util;

#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (
            format!(
                "id={}&last_name=&first_name=ursila&email_address=ursula_le_guin%40gmail.com",
                Uuid::new_v4().to_string()
            ),
            "empty name",
        ),
        (
            format!(
                "id={}&last_name=bob&first_name=ursila&email_address=",
                Uuid::new_v4().to_string()
            ),
            "empty email",
        ),
        (
            format!(
                "id={}&last_name=&first_name=ursila&email_address=not-an-email-address",
                Uuid::new_v4().to_string()
            ),
            "invalid email",
        ),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = Client::new();

    let subscriber = store_subscriber(app.clone()).await;

    let body = format!(
        "id={}&subscriber_id={}&subscription_first_name=joe&\
            subscription_last_name=b&subscription_mailing_address_line_1=123%20Main&\
            subscription_city=Kansas%20City&subscription_state=MO&subscription_postal_code=64108&\
            subscription_email_address=ursula_le_guin%40gmail.com&subscription_type=Electronic",
        Uuid::new_v4().to_string(),
        subscriber.id
    );
    let response = client
        .post(&format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT subscription_first_name, subscription_last_name, subscription_postal_code, id FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.subscription_first_name, "joe");
    assert_eq!(saved.subscription_last_name, "b");
}
