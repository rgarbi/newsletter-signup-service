use once_cell::sync::Lazy;
use reqwest::Response;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use newsletter_signup_service::auth::token::generate_token;
use newsletter_signup_service::configuration::{get_configuration, DatabaseSettings};
use newsletter_signup_service::domain::new_subscriber::{
    OverTheWireCreateSubscriber, OverTheWireSubscriber,
};
use newsletter_signup_service::domain::new_subscription::{
    OverTheWireCreateSubscription, SubscriptionType,
};
use newsletter_signup_service::domain::new_user::{ResetPassword, SignUp};
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
    pub async fn user_signup(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/sign_up", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn login(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/login", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn reset_password(&self, body: String, token: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/reset_password", &self.address))
            .header("Content-Type", "application/json")
            .bearer_auth(token)
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn forgot_password(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/forgot_password", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_subscriber(&self, body: String, token: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscribers", &self.address))
            .header("Content-Type", "application/json")
            .bearer_auth(token)
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_subscriber_by_id(&self, id: String, token: String) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/subscribers/{}", &self.address, id))
            .bearer_auth(token)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn get_subscriber_by_email(&self, email: String, token: String) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/subscribers?email={}", &self.address, email))
            .bearer_auth(token)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn get_subscriber_by_user_id(
        &self,
        user_id: String,
        token: String,
    ) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!(
                "{}/subscribers?user_id={}",
                &self.address, user_id
            ))
            .bearer_auth(token)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn get_subscriber_by_user_id_and_email(
        &self,
        user_id: String,
        email: String,
        token: String,
    ) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!(
                "{}/subscribers?user_id={}&email={}",
                &self.address, user_id, email
            ))
            .bearer_auth(token)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn post_subscription(&self, body: String, token: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/json")
            .bearer_auth(token)
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_subscriptions_by_subscriber_id(
        &self,
        subscriber_id: String,
        token: String,
    ) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!(
                "{}/subscribers/{}/subscriptions",
                &self.address, subscriber_id
            ))
            .bearer_auth(token)
            .send()
            .await
            .expect("Got a subscriber back")
    }

    pub async fn get_subscription_by_id(&self, id: String, token: String) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/subscriptions/{}", &self.address, id))
            .bearer_auth(token)
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
    let response = app
        .post_subscriber(
            subscriber.to_json(),
            generate_token(subscriber.user_id.clone()),
        )
        .await;
    assert_eq!(200, response.status().as_u16());

    app.from_response_to_over_the_wire_subscriber(
        app.get_subscriber_by_email(
            subscriber.email_address.clone(),
            generate_token(subscriber.user_id.clone()),
        )
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

pub fn generate_signup() -> SignUp {
    SignUp {
        email_address: format!("{}@gmail.com", Uuid::new_v4().to_string()),
        password: Uuid::new_v4().to_string(),
        first_name: Uuid::new_v4().to_string(),
        last_name: Uuid::new_v4().to_string(),
    }
}

pub fn generate_reset_password(username: String, old_password: String) -> ResetPassword {
    ResetPassword {
        email_address: username,
        old_password,
        new_password: Uuid::new_v4().to_string(),
    }
}

pub fn generate_over_the_wire_subscriber() -> OverTheWireCreateSubscriber {
    OverTheWireCreateSubscriber {
        first_name: Uuid::new_v4().to_string(),
        last_name: Uuid::new_v4().to_string(),
        email_address: format!("{}@gmail.com", Uuid::new_v4().to_string()),
        user_id: Uuid::new_v4().to_string(),
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
