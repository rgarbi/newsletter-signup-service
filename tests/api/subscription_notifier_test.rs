use claim::assert_ok;
use fake::faker::internet::en::SafeEmail;
use fake::{Fake, Faker};
use newsletter_signup_service::background::new_subscription_notifier::notify_subscriber;
use newsletter_signup_service::db::subscribers_db_broker::{
    insert_subscriber, retrieve_subscriber_by_user_id,
};
use newsletter_signup_service::domain::subscriber_models::NewSubscriber;
use newsletter_signup_service::domain::valid_email::ValidEmail;
use newsletter_signup_service::email_client::EmailClient;
use secrecy::Secret;
use wiremock::matchers::{header, header_exists, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helper::{generate_over_the_wire_subscriber, spawn_app, store_subscription};

#[tokio::test]
async fn notify_of_new_subscriber_works() {
    let app = spawn_app().await;

    let subscriber = generate_over_the_wire_subscriber();
    let new_subscriber: NewSubscriber = subscriber.clone().try_into().unwrap();
    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    let result = insert_subscriber(&new_subscriber, &mut transaction).await;
    assert_ok!(result);
    assert_ok!(transaction.commit().await);

    let stored_subscriber =
        retrieve_subscriber_by_user_id(subscriber.user_id.as_str(), &app.db_pool)
            .await
            .unwrap();
    let stored_subscription =
        store_subscription(stored_subscriber.clone().id.to_string(), None, &app).await;

    Mock::given(header_exists("Authorization"))
        .and(header("Content-Type", "application/json"))
        .and(path("v3/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    notify_subscriber(
        stored_subscription.id.clone(),
        email_client(app.email_server.uri().clone()),
        &app.db_pool,
    )
    .await;
}

fn email_client(base_url: String) -> EmailClient {
    EmailClient::new(
        base_url,
        email(),
        Secret::new(Faker.fake()),
        std::time::Duration::from_millis(200),
    )
}

fn email() -> ValidEmail {
    ValidEmail::parse(SafeEmail().fake()).unwrap()
}
