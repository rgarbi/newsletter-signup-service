use claim::assert_ok;

use newsletter_signup_service::db::users::count_users_with_username;

use crate::helper::{generate_signup, spawn_app};

#[tokio::test]
async fn valid_users_can_create_an_account() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;

    assert_eq!(200, response.status().as_u16());

    let result = count_users_with_username(&signup.username, &app.db_pool).await;
    assert_ok!(&result);
    assert_eq!(1, result.unwrap());
}

#[tokio::test]
async fn signing_up_twice_results_in_conflict() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());
    let result = count_users_with_username(&signup.username, &app.db_pool).await;
    assert_ok!(&result);
    assert_eq!(1, result.unwrap());

    let conflict = app.user_signup(signup.to_json()).await;
    assert_eq!(409, conflict.status().as_u16());
}

#[tokio::test]
async fn sign_up_then_login() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;

    assert_eq!(200, response.status().as_u16());

    let login_response = app.login(signup.to_json()).await;
    assert_eq!(200, login_response.status().as_u16());
}
