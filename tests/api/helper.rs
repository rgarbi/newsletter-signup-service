use cached::proc_macro::once;
use once_cell::sync::Lazy;
use reqwest::Response;
use serde::Deserialize;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use newsletter_signup_service::configuration::{get_configuration, DatabaseSettings};
use newsletter_signup_service::domain::new_subscriber::{
    OverTheWireCreateSubscriber, OverTheWireSubscriber,
};
use newsletter_signup_service::domain::new_subscription::{
    OverTheWireCreateSubscription, SubscriptionType,
};
use newsletter_signup_service::startup::{get_connection_pool, Application};
use newsletter_signup_service::telemetry::{get_subscriber, init_subscriber};

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

#[derive(Clone)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriber(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscribers", &self.address))
            .header("Content-Type", "application/json")
            .bearer_auth(get_bearer_token().await)
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_subscriber_by_id(&self, id: String) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/subscribers/{}", &self.address, id))
            .bearer_auth(get_bearer_token().await)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn get_subscriber_by_email(&self, email: String) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/subscribers?email={}", &self.address, email))
            .bearer_auth(get_bearer_token().await)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/json")
            .bearer_auth(get_bearer_token().await)
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_subscriptions_by_subscriber_id(
        &self,
        subscriber_id: String,
    ) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!(
                "{}/subscribers/{}/subscriptions",
                &self.address, subscriber_id
            ))
            .bearer_auth(get_bearer_token().await)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn get_subscription_by_id(&self, id: String) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/subscriptions/{}", &self.address, id))
            .bearer_auth(get_bearer_token().await)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn from_response_to_over_the_wire_subscriber(
        &self,
        response: Response,
    ) -> OverTheWireSubscriber {
        let response_body = response.text().await.unwrap();
        serde_json::from_str(response_body.as_str()).unwrap()
    }
}

pub async fn store_subscriber(
    app: TestApp,
    subscriber: Option<OverTheWireCreateSubscriber>,
) -> OverTheWireSubscriber {
    let subscriber = subscriber.unwrap_or_else(|| generate_over_the_wire_subscriber());
    let response = app.post_subscriber(subscriber.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    app.from_response_to_over_the_wire_subscriber(
        app.get_subscriber_by_email(subscriber.email_address.clone())
            .await,
    )
    .await
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
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

pub fn generate_over_the_wire_subscriber() -> OverTheWireCreateSubscriber {
    OverTheWireCreateSubscriber {
        first_name: Uuid::new_v4().to_string(),
        last_name: Uuid::new_v4().to_string(),
        email_address: format!("{}@gmail.com", Uuid::new_v4().to_string()),
    }
}

pub fn generate_over_the_wire_subscription(subscriber_id: String) -> OverTheWireCreateSubscription {
    OverTheWireCreateSubscription {
        subscriber_id,
        subscription_type: SubscriptionType::Electronic,
        subscription_state: Uuid::new_v4().to_string(),
        subscription_last_name: Uuid::new_v4().to_string(),
        subscription_city: Uuid::new_v4().to_string(),
        subscription_email_address: format!("{}@gmail.com", Uuid::new_v4().to_string()),
        subscription_postal_code: Uuid::new_v4().to_string(),
        subscription_mailing_address_line_2: Option::from(Uuid::new_v4().to_string()),
        subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
        subscription_first_name: Uuid::new_v4().to_string(),
    }
}

#[derive(Clone, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
}

pub async fn get_bearer_token() -> String {
    call_auth0().await.access_token
}

#[once(time = 500)]
pub async fn call_auth0() -> Token {
    let config = get_configuration()
        .expect("Could not get the config")
        .auth0config;
    let url = format!("https://{}/oauth/token", config.domain);
    let result = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .body(format!(
            r#"
            {{
                "client_id":"{}",
                "client_secret":"{}",
                "audience":"https://hello-world.example.com",
                "grant_type":"client_credentials"
              }}"#,
            std::env::var("AUTH0_CLIENT_ID").unwrap(),
            std::env::var("AUTH0_CLIENT_SECRET").unwrap()
        ))
        .send()
        .await
        .expect("Got a a token back");
    let response_body = result.text().await.unwrap();
    serde_json::from_str(response_body.as_str()).unwrap()
}
