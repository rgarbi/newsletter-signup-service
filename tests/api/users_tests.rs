use claim::assert_ok;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use newsletter_signup_service::auth::token::{generate_token, LoginResponse};
use newsletter_signup_service::db::users::count_users_with_email_address;
use newsletter_signup_service::domain::new_user::{ForgotPassword, LogIn, SignUp};
use newsletter_signup_service::email_client::SendEmailRequest;

use crate::helper::{generate_reset_password, generate_signup, spawn_app};

#[tokio::test]
async fn valid_users_can_create_an_account() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    Mock::given(path("/v3/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let result = count_users_with_email_address(&signup.email_address, &app.db_pool).await;
    assert_ok!(&result);
    assert_eq!(1, result.unwrap());
}

#[tokio::test]
async fn upper_and_lower_case_email_addresses_are_the_same() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let result = count_users_with_email_address(&signup.email_address, &app.db_pool).await;
    assert_ok!(&result);
    assert_eq!(1, result.unwrap());

    let other_signup = SignUp {
        first_name: signup.first_name.clone(),
        last_name: signup.last_name.clone(),
        email_address: signup.email_address.to_uppercase(),
        password: Uuid::new_v4().to_string(),
    };

    let other_response = app.user_signup(other_signup.to_json()).await;
    assert_eq!(409, other_response.status().as_u16());
}

#[tokio::test]
async fn signing_up_twice_results_in_conflict() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());
    let result = count_users_with_email_address(&signup.email_address, &app.db_pool).await;
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

    let log_in = LogIn {
        email_address: signup.email_address,
        password: signup.password,
    };
    let login_response = app.login(log_in.to_json()).await;
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

#[tokio::test]
async fn given_a_valid_sign_up_when_i_sign_up_a_subscriber_is_also_created() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;

    assert_eq!(200, response.status().as_u16());

    let result = count_users_with_email_address(&signup.email_address, &app.db_pool).await;
    assert_ok!(&result);
    assert_eq!(1, result.unwrap());
    let signup_response_body = response.text().await.unwrap();
    let login: LoginResponse = serde_json::from_str(signup_response_body.as_str()).unwrap();

    let subscriber_response = app
        .get_subscriber_by_email(signup.email_address, login.token)
        .await;
    assert_eq!(200, subscriber_response.status().as_u16());
}

#[tokio::test]
async fn forgot_password_works() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let forgot_password = ForgotPassword {
        email_address: signup.email_address,
    };

    Mock::given(path("/v3/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let forgot_password_response = app.forgot_password(forgot_password.to_json()).await;
    assert_eq!(200, forgot_password_response.status().as_u16());
}

#[tokio::test]
async fn forgot_password_no_user_still_gives_200() {
    let app = spawn_app().await;

    let forgot_password = ForgotPassword {
        email_address: format!("{}@GMAIL.COM", Uuid::new_v4().to_string()),
    };

    let forgot_password_response = app.forgot_password(forgot_password.to_json()).await;
    assert_eq!(200, forgot_password_response.status().as_u16());
}

#[tokio::test]
async fn forgot_password_many_times_works() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let forgot_password = ForgotPassword {
        email_address: signup.email_address,
    };

    Mock::given(path("/v3/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    let forgot_password_response = app.forgot_password(forgot_password.to_json()).await;
    assert_eq!(200, forgot_password_response.status().as_u16());

    let forgot_password_response2 = app.forgot_password(forgot_password.to_json()).await;
    assert_eq!(200, forgot_password_response2.status().as_u16());
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;

    let signup = generate_signup();
    let response = app.user_signup(signup.to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let forgot_password = ForgotPassword {
        email_address: signup.email_address,
    };

    Mock::given(path("/v3/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let forgot_password_response = app.forgot_password(forgot_password.to_json()).await;
    assert_eq!(200, forgot_password_response.status().as_u16());

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let body: SendEmailRequest = serde_json::from_slice(&email_request.body).unwrap();

    // Extract the link from one of the request fields.
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let text_link = get_link(body.content[0].value.as_str());
    assert_eq!(false, text_link.is_empty());
}
