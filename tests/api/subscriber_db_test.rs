use claim::{assert_err, assert_ok};
use newsletter_signup_service::db::subscribers_db_broker::{
    insert_subscriber, retrieve_subscriber_by_user_id, set_stripe_customer_id,
};
use uuid::Uuid;

use newsletter_signup_service::db::users::{
    count_users_with_email_address, get_user_by_email_address, get_user_by_user_id, insert_user,
};
use newsletter_signup_service::domain::subscriber_models::NewSubscriber;
use newsletter_signup_service::domain::user_models::SignUp;

use crate::helper::{generate_over_the_wire_subscriber, spawn_app};

#[tokio::test]
async fn set_stripe_customer_id_works() {
    let app = spawn_app().await;

    let subscriber = generate_over_the_wire_subscriber();
    let new_subscriber: NewSubscriber = subscriber.try_into().unwrap();

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
