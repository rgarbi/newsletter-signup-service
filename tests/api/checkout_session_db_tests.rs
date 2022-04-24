use claim::assert_ok;
use newsletter_signup_service::db::checkout_session_db_broker::{
    cancel_checkout_session_by_stripe_session_id, insert_checkout_session,
    retrieve_checkout_session_by_stripe_session_id,
    set_checkout_session_state_to_success_by_stripe_session_id,
};
use newsletter_signup_service::domain::checkout_models::CheckoutSessionState;
use uuid::Uuid;

use crate::helper::{generate_checkout_session, generate_new_subscription, spawn_app};

#[tokio::test]
async fn insert_checkout_session_works() {
    let app = spawn_app().await;

    let stripe_session_id = Uuid::new_v4().to_string();
    let checkout_session = generate_checkout_session(Some(stripe_session_id.clone()));

    let result = insert_checkout_session(
        checkout_session.user_id.clone(),
        checkout_session.price_lookup_key,
        generate_new_subscription(Uuid::new_v4().to_string()),
        stripe_session_id.clone(),
        &app.db_pool,
    )
    .await;
    assert_ok!(result);
}

#[tokio::test]
async fn retrieve_checkout_session_by_stripe_session_id_works() {
    let app = spawn_app().await;

    let stripe_session_id = Uuid::new_v4().to_string();
    let checkout_session = generate_checkout_session(Some(stripe_session_id.clone()));

    let result = insert_checkout_session(
        checkout_session.user_id.clone(),
        checkout_session.price_lookup_key,
        generate_new_subscription(Uuid::new_v4().to_string()),
        stripe_session_id.clone(),
        &app.db_pool,
    )
    .await;
    assert_ok!(result);

    let checkout_session_result =
        retrieve_checkout_session_by_stripe_session_id(&stripe_session_id, &app.db_pool).await;
    assert_ok!(&checkout_session_result);

    let user_id = checkout_session_result.unwrap().user_id;
    assert_eq!(user_id, checkout_session.user_id);
}

#[tokio::test]
async fn cancel_checkout_session_by_stripe_session_id_works() {
    let app = spawn_app().await;

    let stripe_session_id = Uuid::new_v4().to_string();
    let checkout_session = generate_checkout_session(Some(stripe_session_id.clone()));

    let result = insert_checkout_session(
        checkout_session.user_id.clone(),
        checkout_session.price_lookup_key,
        generate_new_subscription(Uuid::new_v4().to_string()),
        stripe_session_id.clone(),
        &app.db_pool,
    )
    .await;
    assert_ok!(result);

    let cancel_session_result =
        cancel_checkout_session_by_stripe_session_id(stripe_session_id.clone(), &app.db_pool).await;
    assert_ok!(&cancel_session_result);

    let checkout_session_result =
        retrieve_checkout_session_by_stripe_session_id(&stripe_session_id, &app.db_pool).await;
    assert_ok!(&checkout_session_result);

    let state = checkout_session_result.unwrap().session_state;
    assert_eq!(state.as_str(), CheckoutSessionState::Cancelled.as_str());
}

#[tokio::test]
async fn set_checkout_session_state_to_success_by_stripe_session_id_works() {
    let app = spawn_app().await;

    let stripe_session_id = Uuid::new_v4().to_string();
    let checkout_session = generate_checkout_session(Some(stripe_session_id.clone()));

    let result = insert_checkout_session(
        checkout_session.user_id.clone(),
        checkout_session.price_lookup_key,
        generate_new_subscription(Uuid::new_v4().to_string()),
        stripe_session_id.clone(),
        &app.db_pool,
    )
    .await;
    assert_ok!(result);

    let mut transaction = app.db_pool.begin().await.unwrap();

    let success_session_result = set_checkout_session_state_to_success_by_stripe_session_id(
        &stripe_session_id.clone(),
        &mut transaction,
    )
    .await;
    assert_ok!(&success_session_result);

    assert_ok!(transaction.commit().await);

    let checkout_session_result =
        retrieve_checkout_session_by_stripe_session_id(&stripe_session_id, &app.db_pool).await;
    assert_ok!(&checkout_session_result);

    let state = checkout_session_result.unwrap().session_state;
    assert_eq!(
        state.as_str(),
        CheckoutSessionState::CompletedSuccessfully.as_str()
    );
}
