use claims::assert_ok;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use newsletter_signup_service::auth::token::{generate_token, LoginResponse};
use newsletter_signup_service::db::users::count_users_with_email_address;
use newsletter_signup_service::domain::user_models::{
    ForgotPassword, LogIn, ResetPasswordFromForgotPassword, SignUp, UserGroup,
};
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
        name: signup.name.clone(),
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
        name: Uuid::new_v4().to_string(),
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
        .reset_password(
            reset_password.to_json(),
            generate_token(login.user_id, UserGroup::USER),
        )
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
        .reset_password(
            reset_password.to_json(),
            generate_token(login.user_id, UserGroup::USER),
        )
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
        .reset_password(
            reset_password.to_json(),
            generate_token(login.user_id, UserGroup::USER),
        )
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
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
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
async fn forgot_password_sends_a_confirmation_email_with_a_link() {
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

#[tokio::test]
async fn forgot_password_given_when_then() {
    let app = spawn_app().await;
    //GIVEN: A user who has forgotten their password and has requested to reset their password
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

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    //WHEN: The user gets the email it contains a link.
    let text_link = get_link(body.content[0].value.as_str());
    assert_eq!(false, text_link.is_empty());

    //THEN: The user can pass that link to the server and get back a token
    let otp_url = url::Url::parse(text_link.as_str()).unwrap();
    let mut query_pairs = otp_url.query_pairs();
    assert_eq!(query_pairs.count(), 1);
    let otp = query_pairs.next().unwrap().1.to_string();
    let response = app.forgot_password_login(otp).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn forgot_password_reset_password_given_when_then() {
    let app = spawn_app().await;
    //GIVEN: A user who has forgotten their password and has requested to reset their password
    let signup = generate_signup();
    let response = app.user_signup(signup.clone().to_json()).await;
    assert_eq!(200, response.status().as_u16());

    let forgot_password = ForgotPassword {
        email_address: signup.clone().email_address,
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

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    //WHEN: The user gets the email it contains a link.
    let text_link = get_link(body.content[0].value.as_str());
    assert_eq!(false, text_link.is_empty());

    //THEN: The user can pass that link to the server and get back a token which can be used to reset their password
    let otp_url = url::Url::parse(text_link.as_str()).unwrap();
    let mut query_pairs = otp_url.query_pairs();
    assert_eq!(query_pairs.count(), 1);
    let otp = query_pairs.next().unwrap().1.to_string();
    let response = app.forgot_password_login(otp).await;

    assert_eq!(&200, &response.status().as_u16());

    let forgot_password_login_response_body = response.text().await.unwrap();
    let login: LoginResponse =
        serde_json::from_str(forgot_password_login_response_body.as_str()).unwrap();

    let reset = ResetPasswordFromForgotPassword {
        user_id: login.user_id,
        new_password: Uuid::new_v4().to_string(),
    };

    let reset_password_response = app
        .forgot_password_rest_password(reset.to_json(), login.token)
        .await;
    assert_eq!(&200, &reset_password_response.status().as_u16());

    let login: LogIn = LogIn {
        email_address: signup.email_address,
        password: reset.clone().new_password,
    };

    let login_response = app.login(login.to_json()).await;
    assert_eq!(&200, &login_response.status().as_u16());
}

#[tokio::test]
async fn check_token_works() {
    let app = spawn_app().await;

    let user_id = Uuid::new_v4().to_string();
    let user_response = app
        .check_token(
            user_id.clone(),
            generate_token(user_id.clone(), UserGroup::USER),
        )
        .await;
    assert_eq!(&200, &user_response.status().as_u16());

    let admin_response = app
        .check_token(
            user_id.clone(),
            generate_token(user_id.clone(), UserGroup::ADMIN),
        )
        .await;
    assert_eq!(&200, &admin_response.status().as_u16());
}

#[tokio::test]
async fn check_token_does_not_work() {
    let app = spawn_app().await;

    let user_id = Uuid::new_v4().to_string();
    let response = app
        .check_token(
            user_id.clone(),
            generate_token(Uuid::new_v4().to_string(), UserGroup::USER),
        )
        .await;
    assert_eq!(&401, &response.status().as_u16());
}

#[tokio::test]
async fn check_admin_token_works() {
    let app = spawn_app().await;

    let user_id = Uuid::new_v4().to_string();
    let admin_response = app
        .check_admin_token(
            user_id.clone(),
            generate_token(user_id.clone(), UserGroup::ADMIN),
        )
        .await;
    assert_eq!(&200, &admin_response.status().as_u16());

    let admin_response = app
        .check_token(
            user_id.clone(),
            generate_token(user_id.clone(), UserGroup::ADMIN),
        )
        .await;
    assert_eq!(&200, &admin_response.status().as_u16());
}

#[tokio::test]
async fn check_admin_token_does_not_work() {
    let app = spawn_app().await;

    let user_id = Uuid::new_v4().to_string();
    assert_eq!(
        401,
        app.check_admin_token(
            user_id.clone(),
            generate_token(user_id.clone(), UserGroup::USER),
        )
        .await
        .status()
        .as_u16()
    );

    assert_eq!(
        401,
        app.check_admin_token(
            Uuid::new_v4().to_string(),
            generate_token(user_id.clone(), UserGroup::USER),
        )
        .await
        .status()
        .as_u16()
    );

    assert_eq!(
        401,
        app.check_admin_token(
            Uuid::new_v4().to_string(),
            generate_token(user_id.clone(), UserGroup::ADMIN),
        )
        .await
        .status()
        .as_u16()
    );
}
