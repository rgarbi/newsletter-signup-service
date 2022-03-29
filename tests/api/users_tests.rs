use claim::assert_ok;
use uuid::Uuid;

use newsletter_signup_service::auth::token::{generate_token, LoginResponse};
use newsletter_signup_service::db::users::count_users_with_username;
use newsletter_signup_service::domain::new_user::SignUp;

use crate::helper::{generate_reset_password, generate_signup, spawn_app};

#[tokio::test]
async fn valid_users_can_create_an_account() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;

    assert_eq!(200, response.status().as_u16());

    let result = count_users_with_username(&signup.email_address, &app.db_pool).await;
    assert_ok!(&result);
    assert_eq!(1, result.unwrap());
}

#[tokio::test]
async fn signing_up_twice_results_in_conflict() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());
    let result = count_users_with_username(&signup.email_address, &app.db_pool).await;
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

#[tokio::test]
async fn login_with_a_bad_username_gives_a_400() {
    let app = spawn_app().await;

    let signup = generate_signup();

    let login_response = app.login(signup.to_json()).await;
    assert_eq!(400, login_response.status().as_u16());
}

#[tokio::test]
async fn login_with_a_bad_password_gives_a_400() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let bad_password = SignUp {
        email_address: signup.email_address,
        password: Uuid::new_v4().to_string(),
        first_name: Uuid::new_v4().to_string(),
        last_name: Uuid::new_v4().to_string(),
    };

    let login_response = app.login(bad_password.to_json()).await;
    assert_eq!(400, login_response.status().as_u16());
}

#[tokio::test]
async fn sign_up_then_reset_password() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let login_response = app.login(signup.to_json()).await;
    assert_eq!(200, login_response.status().as_u16());
    let login_response_body = login_response.text().await.unwrap();
    let login: LoginResponse = serde_json::from_str(login_response_body.as_str()).unwrap();

    let reset_password = generate_reset_password(signup.email_address, signup.password);
    let reset_password_response = app
        .reset_password(reset_password.to_json(), generate_token(login.user_id))
        .await;
    assert_eq!(200, reset_password_response.status().as_u16());
}

#[tokio::test]
async fn reset_password_with_wrong_username_gives_a_400() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());
    let signup_response_body = response.text().await.unwrap();
    let login: LoginResponse = serde_json::from_str(signup_response_body.as_str()).unwrap();

    let reset_password = generate_reset_password(Uuid::new_v4().to_string(), signup.password);
    let reset_password_response = app
        .reset_password(reset_password.to_json(), generate_token(login.user_id))
        .await;
    assert_eq!(400, reset_password_response.status().as_u16());
}

#[tokio::test]
async fn reset_password_with_wrong_password_gives_a_400() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());
    let signup_response_body = response.text().await.unwrap();
    let login: LoginResponse = serde_json::from_str(signup_response_body.as_str()).unwrap();

    let reset_password = generate_reset_password(signup.email_address, Uuid::new_v4().to_string());
    let reset_password_response = app
        .reset_password(reset_password.to_json(), generate_token(login.user_id))
        .await;
    assert_eq!(400, reset_password_response.status().as_u16());
}

#[tokio::test]
async fn reset_password_with_wrong_token_gives_a_401() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let reset_password = generate_reset_password(signup.email_address, Uuid::new_v4().to_string());
    let reset_password_response = app
        .reset_password(
            reset_password.to_json(),
            generate_token(Uuid::new_v4().to_string()),
        )
        .await;
    assert_eq!(401, reset_password_response.status().as_u16());
}
