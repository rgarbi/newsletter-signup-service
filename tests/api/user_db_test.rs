use claims::{assert_err, assert_ok};
use uuid::Uuid;

use newsletter_signup_service::db::users::{
    count_users_with_email_address, get_user_by_email_address, get_user_by_user_id, insert_user,
    update_password,
};
use newsletter_signup_service::domain::user_models::{SignUp, UserGroup};

use crate::helper::spawn_app;

#[tokio::test]
async fn insert_user_works() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        email_address: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        name: Uuid::new_v4().to_string(),
    };

    let mut transaction = app.db_pool.clone().begin().await.unwrap();

    let result = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::USER,
        &mut transaction,
    )
    .await;
    assert_ok!(result);
    assert_ok!(transaction.commit().await);
}

#[tokio::test]
async fn insert_admin_user_works() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        email_address: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        name: Uuid::new_v4().to_string(),
    };

    let mut transaction = app.db_pool.clone().begin().await.unwrap();

    let result = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::ADMIN,
        &mut transaction,
    )
    .await;
    assert_ok!(&result);
    assert_ok!(transaction.commit().await);

    let user_id = result.unwrap();
    assert_ok!(get_user_by_user_id(user_id.as_str(), &app.db_pool).await);
}

#[tokio::test]
async fn insert_user_two_times_does_not_work() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        email_address: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        name: Uuid::new_v4().to_string(),
    };

    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    let result = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::USER,
        &mut transaction,
    )
    .await;
    assert_ok!(result);

    let err = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::USER,
        &mut transaction,
    )
    .await;
    assert_err!(&err);
    println!("{:?}", err);
}

#[tokio::test]
async fn count_users() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        email_address: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        name: Uuid::new_v4().to_string(),
    };

    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    let result = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::USER,
        &mut transaction,
    )
    .await;
    assert_ok!(result);

    assert_ok!(transaction.commit().await);

    let ok =
        count_users_with_email_address(&Uuid::new_v4().to_string(), &app.db_pool.clone()).await;
    assert_ok!(&ok);
    assert_eq!(0, ok.unwrap());

    let ok_with_one =
        count_users_with_email_address(&sign_up.email_address, &app.db_pool.clone()).await;
    assert_ok!(&ok_with_one);
    assert_eq!(1, ok_with_one.unwrap());
}

#[tokio::test]
async fn count_users_fails() {
    let app = spawn_app().await;

    sqlx::query!("DROP TABLE users CASCADE")
        .execute(&app.db_pool)
        .await
        .expect("Failed to drop.");

    let err =
        count_users_with_email_address(&Uuid::new_v4().to_string(), &app.db_pool.clone()).await;
    assert_err!(&err);
}

#[tokio::test]
async fn get_user_by_username_test() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        email_address: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        name: Uuid::new_v4().to_string(),
    };

    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    let result = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::USER,
        &mut transaction,
    )
    .await;
    assert_ok!(result);
    assert_ok!(transaction.commit().await);

    assert_ok!(get_user_by_email_address(&sign_up.email_address, &app.db_pool).await);
}

#[tokio::test]
async fn get_user_by_username_not_found_test() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        email_address: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        name: Uuid::new_v4().to_string(),
    };

    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    let result = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::USER,
        &mut transaction,
    )
    .await;
    assert_ok!(result);

    assert_err!(get_user_by_email_address(&Uuid::new_v4().to_string(), &app.db_pool).await);
}

#[tokio::test]
async fn get_user_by_user_id_test() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        email_address: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        name: Uuid::new_v4().to_string(),
    };

    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    let result = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::USER,
        &mut transaction,
    )
    .await;
    assert_ok!(result);
    assert_ok!(transaction.commit().await);

    let get_by_email_result = get_user_by_email_address(&sign_up.email_address, &app.db_pool).await;
    assert_ok!(&get_by_email_result);

    let saved_user = get_by_email_result.unwrap();

    let get_by_user_id_result =
        get_user_by_user_id(saved_user.user_id.to_string().as_str(), &app.db_pool).await;
    assert_ok!(&get_by_user_id_result);

    let saved_user_by_id = get_by_user_id_result.unwrap();
    assert_eq!(saved_user.user_id, saved_user_by_id.user_id)
}

#[tokio::test]
async fn get_user_by_user_id_fails() {
    let app = spawn_app().await;

    sqlx::query!("DROP TABLE users CASCADE")
        .execute(&app.db_pool)
        .await
        .expect("Failed to drop.");

    let get_by_user_id_result =
        get_user_by_user_id(Uuid::new_v4().to_string().as_str(), &app.db_pool).await;
    assert_err!(&get_by_user_id_result);
}

#[tokio::test]
async fn update_password_test() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        email_address: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        name: Uuid::new_v4().to_string(),
    };

    let mut transaction = app.db_pool.clone().begin().await.unwrap();
    let result = insert_user(
        &sign_up.email_address,
        &sign_up.password,
        UserGroup::USER,
        &mut transaction,
    )
    .await;
    assert_ok!(result);
    assert_ok!(transaction.commit().await);

    assert_ok!(get_user_by_email_address(&sign_up.email_address, &app.db_pool).await);

    let update_result =
        update_password(&sign_up.email_address.as_str(), "newpassword", &app.db_pool).await;
    assert_ok!(update_result);
}

#[tokio::test]
async fn update_password_failed() {
    let app = spawn_app().await;

    sqlx::query!("DROP TABLE users CASCADE")
        .execute(&app.db_pool)
        .await
        .expect("Failed to drop.");

    let update_result = update_password(
        &Uuid::new_v4().to_string().as_str(),
        "newpassword",
        &app.db_pool,
    )
    .await;
    assert_err!(update_result);
}
