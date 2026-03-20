use claims::assert_ok;
use newsletter_signup_service::db::subscribers_db_broker::{
    insert_subscriber, retrieve_all_subscribers, retrieve_subscriber_by_user_id,
    set_stripe_customer_id,
};
use newsletter_signup_service::domain::subscriber_models::NewSubscriber;
use uuid::Uuid;

use crate::helper::{generate_over_the_wire_subscriber, spawn_app};

#[tokio::test]
async fn retrieve_all_subscribers_returns_empty_when_no_subscribers() {
    let app = spawn_app().await;

    let result = retrieve_all_subscribers(&app.db_pool).await;
    assert_ok!(&result);

    let subscribers = result.unwrap();
    assert!(subscribers.is_empty());
}

#[tokio::test]
async fn retrieve_all_subscribers_returns_single_subscriber() {
    let app = spawn_app().await;

    let subscriber = generate_over_the_wire_subscriber();
    let new_subscriber: NewSubscriber = subscriber.clone().try_into().unwrap();

    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    assert_ok!(insert_subscriber(&new_subscriber, &mut transaction).await);
    assert_ok!(transaction.commit().await);

    let result = retrieve_all_subscribers(&app.db_pool).await;
    assert_ok!(&result);

    let subscribers = result.unwrap();
    assert_eq!(subscribers.len(), 1);
    assert_eq!(subscribers[0].email_address, subscriber.email_address);
    assert_eq!(subscribers[0].name, subscriber.name);
    assert_eq!(subscribers[0].user_id, subscriber.user_id);
}

#[tokio::test]
async fn retrieve_all_subscribers_returns_multiple_subscribers() {
    let app = spawn_app().await;

    let subscriber1 = generate_over_the_wire_subscriber();
    let new_subscriber1: NewSubscriber = subscriber1.clone().try_into().unwrap();

    let subscriber2 = generate_over_the_wire_subscriber();
    let new_subscriber2: NewSubscriber = subscriber2.clone().try_into().unwrap();

    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    assert_ok!(insert_subscriber(&new_subscriber1, &mut transaction).await);
    assert_ok!(insert_subscriber(&new_subscriber2, &mut transaction).await);
    assert_ok!(transaction.commit().await);

    let result = retrieve_all_subscribers(&app.db_pool).await;
    assert_ok!(&result);

    let subscribers = result.unwrap();
    assert_eq!(subscribers.len(), 2);

    let emails: Vec<_> = subscribers.iter().map(|s| s.email_address.as_str()).collect();
    assert!(emails.contains(&subscriber1.email_address.as_str()));
    assert!(emails.contains(&subscriber2.email_address.as_str()));
}

#[tokio::test]
async fn set_stripe_customer_id_works() {
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

    let stripe_customer_id = Uuid::new_v4();
    let set_stripe_id_result = set_stripe_customer_id(
        &stored_subscriber.id,
        &stripe_customer_id.to_string(),
        &app.db_pool,
    )
    .await;
    assert_ok!(&set_stripe_id_result);

    let stored_subscriber_with_stripe_customer_id =
        retrieve_subscriber_by_user_id(subscriber.user_id.as_str(), &app.db_pool)
            .await
            .unwrap();

    assert_eq!(
        stripe_customer_id.to_string(),
        stored_subscriber_with_stripe_customer_id
            .stripe_customer_id
            .unwrap()
    )
}
