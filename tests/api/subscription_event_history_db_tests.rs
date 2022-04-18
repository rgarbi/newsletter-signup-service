use claim::assert_ok;

use newsletter_signup_service::db::subscription_history_db_broker::{
    insert_subscription_history_event, retrieve_subscription_events_by_subscription_id,
};
use newsletter_signup_service::domain::subscription_history_models::HistoryEventType;

use crate::helper::{generate_over_the_wire_subscription, spawn_app};

#[tokio::test]
async fn insert_history_works() {
    let app = spawn_app().await;

    let subscription = generate_over_the_wire_subscription();

    let result = insert_subscription_history_event(
        subscription,
        HistoryEventType::UpdatedSubscriptionInformation,
        &app.db_pool,
    )
    .await;
    assert_ok!(result);
}

#[tokio::test]
async fn insert_history_of_each_type_works() {
    let app = spawn_app().await;

    assert_ok!(
        insert_subscription_history_event(
            generate_over_the_wire_subscription(),
            HistoryEventType::UpdatedSubscriptionInformation,
            &app.db_pool,
        )
        .await
    );

    assert_ok!(
        insert_subscription_history_event(
            generate_over_the_wire_subscription(),
            HistoryEventType::Created,
            &app.db_pool,
        )
        .await
    );

    assert_ok!(
        insert_subscription_history_event(
            generate_over_the_wire_subscription(),
            HistoryEventType::ChangedPaymentMethod,
            &app.db_pool,
        )
        .await
    );

    assert_ok!(
        insert_subscription_history_event(
            generate_over_the_wire_subscription(),
            HistoryEventType::Cancelled,
            &app.db_pool,
        )
        .await
    );
}

#[tokio::test]
async fn get_event_history_by_subscription_id_works() {
    let app = spawn_app().await;
    let subscription = generate_over_the_wire_subscription();

    let result = insert_subscription_history_event(
        subscription.clone(),
        HistoryEventType::UpdatedSubscriptionInformation,
        &app.db_pool,
    )
    .await;
    assert_ok!(result);

    let rows_result =
        retrieve_subscription_events_by_subscription_id(subscription.id, &app.db_pool).await;
    assert_ok!(&rows_result);

    assert_eq!(1, rows_result.unwrap().len());
}

#[tokio::test]
async fn get_event_history_by_subscription_id_many_works() {
    let app = spawn_app().await;
    let subscription = generate_over_the_wire_subscription();

    let count = 100;
    for _ in 0..count {
        let result = insert_subscription_history_event(
            subscription.clone(),
            HistoryEventType::UpdatedSubscriptionInformation,
            &app.db_pool,
        )
        .await;
        assert_ok!(result);
    }

    let rows_result =
        retrieve_subscription_events_by_subscription_id(subscription.id, &app.db_pool).await;
    assert_ok!(&rows_result);

    assert_eq!(count, rows_result.unwrap().len());
}
