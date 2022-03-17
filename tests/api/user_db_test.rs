use claim::{assert_err, assert_ok};
use newsletter_signup_service::db::users::{count_users_with_username, insert_user};
use uuid::Uuid;

use newsletter_signup_service::domain::new_user::SignUp;

use crate::helper::spawn_app;

#[tokio::test]
async fn insert_user_works() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        username: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
    };

    let result = insert_user(&sign_up, &app.db_pool.clone()).await;
    assert_ok!(result);
}

#[tokio::test]
async fn insert_user_two_times_does_not_work() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        username: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
    };

    let result = insert_user(&sign_up, &app.db_pool.clone()).await;
    assert_ok!(result);

    let err = insert_user(&sign_up, &app.db_pool.clone()).await;
    assert_err!(&err);
    println!("{:?}", err);
}

#[tokio::test]
async fn count_users() {
    let app = spawn_app().await;

    let sign_up = SignUp {
        username: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
    };

    let result = insert_user(&sign_up, &app.db_pool.clone()).await;
    assert_ok!(result);

    let ok = count_users_with_username(&Uuid::new_v4().to_string(), &app.db_pool.clone()).await;
    assert_ok!(&ok);
    assert_eq!(0, ok.unwrap());
}
