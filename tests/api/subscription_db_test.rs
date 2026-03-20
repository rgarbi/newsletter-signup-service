use chrono::Utc;
use claims::{assert_err, assert_ok};
use newsletter_signup_service::db::subscribers_db_broker::{
    insert_subscriber, retrieve_subscriber_by_user_id,
};
use newsletter_signup_service::db::subscriptions_db_broker::{
    insert_subscription, retrieve_all_subscriptions, retrieve_subscription_by_subscription_id,
    update_subscription_by_subscription_id,
};
use newsletter_signup_service::domain::subscriber_models::NewSubscriber;
use newsletter_signup_service::domain::subscription_models::{
    OverTheWireSubscription, SubscriptionType,
};
use uuid::Uuid;

use crate::helper::{generate_new_subscription, generate_over_the_wire_subscriber, spawn_app};

#[tokio::test]
async fn update_subscription_by_subscription_id_works() {
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

    transaction = app.db_pool.clone().begin().await.unwrap();
    let subscription = generate_new_subscription(stored_subscriber.id.to_string());
    let insert_subscription_result =
        insert_subscription(subscription, Uuid::new_v4().to_string(), &mut transaction).await;
    assert_ok!(&insert_subscription_result);
    assert_ok!(transaction.commit().await);

    let subscription = insert_subscription_result.unwrap();

    let updates = OverTheWireSubscription {
        id: subscription.id.clone(),
        subscriber_id: subscription.subscriber_id.clone(),
        subscription_name: Uuid::new_v4().to_string(),
        subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
        subscription_mailing_address_line_2: Uuid::new_v4().to_string(),
        subscription_city: Uuid::new_v4().to_string(),
        subscription_state: Uuid::new_v4().to_string(),
        subscription_postal_code: Uuid::new_v4().to_string(),
        subscription_email_address: format!("{}@gmail.com", Uuid::new_v4().to_string()),
        subscription_creation_date: Utc::now(),
        subscription_cancelled_on_date: None,
        subscription_anniversary_day: 0,
        subscription_anniversary_month: 0,
        subscription_renewal_date: "".to_string(),
        active: false,
        subscription_type: SubscriptionType::Digital,
        stripe_subscription_id: subscription.stripe_subscription_id.clone(),
    };

    let update_subscription_result =
        update_subscription_by_subscription_id(subscription.id, updates.clone(), &app.db_pool)
            .await;
    assert_ok!(update_subscription_result);

    let updated_subscription_retrieval_result =
        retrieve_subscription_by_subscription_id(subscription.id.clone(), &app.db_pool).await;
    assert_ok!(&updated_subscription_retrieval_result);

    assert_eq!(
        updates.clone().subscription_name,
        updated_subscription_retrieval_result
            .unwrap()
            .subscription_name
    );
}

#[tokio::test]
async fn update_subscription_by_subscription_id_err() {
    let app = spawn_app().await;

    let updates = OverTheWireSubscription {
        id: Uuid::new_v4(),
        subscriber_id: Uuid::new_v4(),
        subscription_name: Uuid::new_v4().to_string(),
        subscription_mailing_address_line_1: Uuid::new_v4().to_string(),
        subscription_mailing_address_line_2: Uuid::new_v4().to_string(),
        subscription_city: Uuid::new_v4().to_string(),
        subscription_state: Uuid::new_v4().to_string(),
        subscription_postal_code: Uuid::new_v4().to_string(),
        subscription_email_address: format!("{}@gmail.com", Uuid::new_v4().to_string()),
        subscription_creation_date: Utc::now(),
        subscription_cancelled_on_date: None,
        subscription_anniversary_day: 0,
        subscription_anniversary_month: 0,
        subscription_renewal_date: "".to_string(),
        active: false,
        subscription_type: SubscriptionType::Digital,
        stripe_subscription_id: Uuid::new_v4().to_string(),
    };

    // Sabotage the database
    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN subscription_name;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let update_subscription_result =
        update_subscription_by_subscription_id(updates.id.clone(), updates.clone(), &app.db_pool)
            .await;
    assert_err!(update_subscription_result);
}

#[tokio::test]
async fn retrieve_all_subscriptions_returns_empty_when_no_subscriptions() {
    let app = spawn_app().await;

    let result = retrieve_all_subscriptions(&app.db_pool).await;
    assert_ok!(&result);

    let subscriptions = result.unwrap();
    assert!(subscriptions.is_empty());
}

#[tokio::test]
async fn retrieve_all_subscriptions_returns_single_subscription() {
    let app = spawn_app().await;

    let subscriber = generate_over_the_wire_subscriber();
    let new_subscriber: NewSubscriber = subscriber.clone().try_into().unwrap();
    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    assert_ok!(insert_subscriber(&new_subscriber, &mut transaction).await);
    assert_ok!(transaction.commit().await);

    let stored_subscriber =
        retrieve_subscriber_by_user_id(subscriber.user_id.as_str(), &app.db_pool)
            .await
            .unwrap();

    let subscription = generate_new_subscription(stored_subscriber.id.to_string());
    let subscription_name = subscription.subscription_name.as_ref().to_string();
    let subscription_email = subscription.subscription_email_address.as_ref().to_string();
    transaction = app.db_pool.clone().begin().await.unwrap();
    let insert_result =
        insert_subscription(subscription, Uuid::new_v4().to_string(), &mut transaction).await;
    assert_ok!(&insert_result);
    assert_ok!(transaction.commit().await);

    let result = retrieve_all_subscriptions(&app.db_pool).await;
    assert_ok!(&result);

    let subscriptions = result.unwrap();
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].subscription_name, subscription_name);
    assert_eq!(subscriptions[0].subscription_email_address, subscription_email);
    assert_eq!(subscriptions[0].subscriber_id, stored_subscriber.id);
}

#[tokio::test]
async fn retrieve_all_subscriptions_returns_multiple_subscriptions() {
    let app = spawn_app().await;

    let subscriber = generate_over_the_wire_subscriber();
    let new_subscriber: NewSubscriber = subscriber.clone().try_into().unwrap();
    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    assert_ok!(insert_subscriber(&new_subscriber, &mut transaction).await);
    assert_ok!(transaction.commit().await);

    let stored_subscriber =
        retrieve_subscriber_by_user_id(subscriber.user_id.as_str(), &app.db_pool)
            .await
            .unwrap();

    let expected_count = 3;
    transaction = app.db_pool.clone().begin().await.unwrap();
    for _ in 0..expected_count {
        let subscription = generate_new_subscription(stored_subscriber.id.to_string());
        assert_ok!(insert_subscription(
            subscription,
            Uuid::new_v4().to_string(),
            &mut transaction
        )
        .await);
    }
    assert_ok!(transaction.commit().await);

    let result = retrieve_all_subscriptions(&app.db_pool).await;
    assert_ok!(&result);

    let subscriptions = result.unwrap();
    assert_eq!(subscriptions.len(), expected_count);
}

#[tokio::test]
async fn get_all_subscriptions_works() {
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
    let expected_number_of_subscriptions = 10;
    transaction = app.db_pool.clone().begin().await.unwrap();

    for _i in 0..expected_number_of_subscriptions {
        let subscription = generate_new_subscription(stored_subscriber.id.to_string());
        let insert_subscription_result =
            insert_subscription(subscription, Uuid::new_v4().to_string(), &mut transaction).await;
        assert_ok!(&insert_subscription_result);
        let _subscription = insert_subscription_result.unwrap();
    }
    assert_ok!(transaction.commit().await);

    let get_all_result = retrieve_all_subscriptions(&app.db_pool).await;
    assert_ok!(&get_all_result);
    assert_eq!(
        expected_number_of_subscriptions,
        get_all_result.unwrap().len()
    )
}

#[tokio::test]
async fn retrieve_all_subscriptions_returns_subscriptions_from_multiple_subscribers() {
    let app = spawn_app().await;

    let subscriber1 = generate_over_the_wire_subscriber();
    let new_subscriber1: NewSubscriber = subscriber1.clone().try_into().unwrap();
    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    assert_ok!(insert_subscriber(&new_subscriber1, &mut transaction).await);
    assert_ok!(transaction.commit().await);

    let subscriber2 = generate_over_the_wire_subscriber();
    let new_subscriber2: NewSubscriber = subscriber2.clone().try_into().unwrap();
    transaction = app.db_pool.clone().begin().await.unwrap();
    assert_ok!(insert_subscriber(&new_subscriber2, &mut transaction).await);
    assert_ok!(transaction.commit().await);

    let stored_subscriber1 =
        retrieve_subscriber_by_user_id(subscriber1.user_id.as_str(), &app.db_pool)
            .await
            .unwrap();
    let stored_subscriber2 =
        retrieve_subscriber_by_user_id(subscriber2.user_id.as_str(), &app.db_pool)
            .await
            .unwrap();

    transaction = app.db_pool.clone().begin().await.unwrap();
    let sub1 = generate_new_subscription(stored_subscriber1.id.to_string());
    let sub2 = generate_new_subscription(stored_subscriber2.id.to_string());
    assert_ok!(insert_subscription(sub1, Uuid::new_v4().to_string(), &mut transaction).await);
    assert_ok!(insert_subscription(sub2, Uuid::new_v4().to_string(), &mut transaction).await);
    assert_ok!(transaction.commit().await);

    let result = retrieve_all_subscriptions(&app.db_pool).await;
    assert_ok!(&result);

    let subscriptions = result.unwrap();
    assert_eq!(subscriptions.len(), 2);

    let subscriber_ids: Vec<_> = subscriptions.iter().map(|s| s.subscriber_id).collect();
    assert!(subscriber_ids.contains(&stored_subscriber1.id));
    assert!(subscriber_ids.contains(&stored_subscriber2.id));
}

#[tokio::test]
async fn retrieve_all_subscriptions_returns_err_on_database_error() {
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
    let expected_number_of_subscriptions = 10;
    transaction = app.db_pool.clone().begin().await.unwrap();

    for _i in 0..expected_number_of_subscriptions {
        let subscription = generate_new_subscription(stored_subscriber.id.to_string());
        let insert_subscription_result =
            insert_subscription(subscription, Uuid::new_v4().to_string(), &mut transaction).await;
        assert_ok!(&insert_subscription_result);
        let _subscription = insert_subscription_result.unwrap();
    }
    assert_ok!(transaction.commit().await);

    // Sabotage the database
    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN subscription_name;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let get_all_result = retrieve_all_subscriptions(&app.db_pool).await;
    assert_err!(&get_all_result);
}
