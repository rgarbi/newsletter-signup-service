use std::net::TcpListener;

use once_cell::sync::Lazy;
use reqwest::Client;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use newsletter_signup_service::configuration::{get_configuration, DatabaseSettings};
use newsletter_signup_service::domain::new_subscriber::OverTheWireSubscriber;
use newsletter_signup_service::startup::run;
use newsletter_signup_service::telemetry::{get_subscriber, init_subscriber};

#[derive(Clone)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

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
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

pub async fn store_subscriber(app: TestApp) -> OverTheWireSubscriber {
    let client = Client::new();

    let first_name = Uuid::new_v4().to_string();
    let body = format!(
        "last_name={}&first_name={}&email_address={}%40gmail.com",
        Uuid::new_v4().to_string(),
        first_name,
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
        "SELECT email_address, first_name, last_name, id FROM subscribers WHERE first_name = $1",
        first_name
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");
    OverTheWireSubscriber {
        id: saved.id.to_string(),
        last_name: saved.last_name,
        email_address: saved.email_address,
        first_name: saved.first_name,
    }
}
