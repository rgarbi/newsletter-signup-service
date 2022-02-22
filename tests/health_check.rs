use std::net::TcpListener;
use std::str::FromStr;

use reqwest::Client;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use newsletter_signup_service::configuration::{get_configuration, DatabaseSettings};
use newsletter_signup_service::routes::Subscriber;
use newsletter_signup_service::startup::run;

#[derive(Clone)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client: Client = Client::new();

    let response = client
        .get(&format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscriptions_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = Client::new();

    let subscriber_id = Uuid::new_v4();
    store_subscriber(subscriber_id, app.clone()).await;

    let body = format!(
        "id={}&subscriber_id={}&subscription_first_name=joe&\
            subscription_last_name=b&subscription_mailing_address_line_1=123%20Main&\
            subscription_city=Kansas%20City&subscription_state=MO&subscription_postal_code=64108&\
            subscription_email_address=ursula_le_guin%40gmail.com&subscription_type=Electronic",
        Uuid::new_v4().to_string(),
        subscriber_id
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
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
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
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let listener: TcpListener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port");

    let connection_pool = configure_database(&configuration.database).await;
    let address = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());

    let server = run(listener, connection_pool.clone()).expect("Failed to start the server");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

pub async fn store_subscriber(id: Uuid, app: TestApp) -> Subscriber {
    let client = Client::new();

    let body = format!(
        "id={}&last_name={}&first_name={}&email_address={}%40gmail.com",
        id,
        Uuid::new_v4().to_string(),
        Uuid::new_v4().to_string(),
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

    let saved = sqlx::query!(
        "SELECT email_address, first_name, last_name, id FROM subscribers WHERE id = $1",
        id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");
    Subscriber {
        id: saved.id.to_string(),
        last_name: saved.last_name,
        email_address: saved.email_address,
        first_name: saved.first_name,
    }
}
