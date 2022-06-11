use chrono::Utc;
use claim::{assert_err, assert_ok};
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
async fn get_all_subscriptions_errors() {
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
