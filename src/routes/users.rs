use actix_web::{web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::password_hashing::{hash_password, validate_password};
use crate::auth::token::{generate_token, get_expires_at, Claims, LoginResponse};
use crate::db::otp_db_broker::insert_otp;
use crate::db::subscribers::insert_subscriber;
use crate::db::users::{
    count_users_with_email_address, get_user_by_email_address, insert_user, update_password,
};
use crate::domain::new_subscriber::NewSubscriber;
use crate::domain::new_user::{ForgotPassword, LogIn, ResetPassword, SignUp};
use crate::domain::otp_models::OneTimePasscode;
use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;
use crate::util::standardize_email;

impl TryFrom<SignUp> for NewSubscriber {
    type Error = String;
    fn try_from(sign_up: SignUp) -> Result<Self, Self::Error> {
        let first_name = ValidName::parse(sign_up.first_name)?;
        let last_name = ValidName::parse(sign_up.last_name)?;
        let email_address = ValidEmail::parse(standardize_email(&sign_up.email_address))?;
        Ok(NewSubscriber {
            first_name,
            last_name,
            email_address,
            user_id: String::new(),
        })
    }
}

#[tracing::instrument(
    name = "Singing up a new user",
    skip(sign_up, pool),
    fields(
        user_username = %sign_up.email_address,
    )
)]
pub async fn sign_up(sign_up: web::Json<SignUp>, pool: web::Data<PgPool>) -> impl Responder {
    let transformed_email = standardize_email(&sign_up.email_address.clone());
    match count_users_with_email_address(&transformed_email, &pool).await {
        Ok(count) => {
            if count > 0 {
                return HttpResponse::Conflict().finish();
            }

            let mut new_subscriber: NewSubscriber = match sign_up.clone().try_into() {
                Ok(subscriber) => subscriber,
                Err(_) => return HttpResponse::BadRequest().finish(),
            };

            let mut transaction = match pool.begin().await {
                Ok(transaction) => transaction,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

            let hashed_password = hash_password(sign_up.clone().password).await;
            let login_response =
                match insert_user(&transformed_email, &hashed_password, &mut transaction).await {
                    Ok(user_id) => LoginResponse {
                        user_id: user_id.clone(),
                        token: generate_token(user_id),
                        expires_on: get_expires_at(Option::None),
                    },
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                };

            new_subscriber.user_id = login_response.user_id.clone();
            match insert_subscriber(&new_subscriber, &mut transaction).await {
                Ok(_) => {
                    if transaction.commit().await.is_err() {
                        HttpResponse::InternalServerError().finish();
                    }
                    HttpResponse::Ok().json(&login_response)
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Login user",
    skip(log_in, pool),
    fields(
        user_username = %log_in.email_address,
    )
)]
pub async fn login(log_in: web::Json<LogIn>, pool: web::Data<PgPool>) -> impl Responder {
    let transformed_email = standardize_email(&log_in.email_address.clone());
    match get_user_by_email_address(&transformed_email, &pool).await {
        Ok(user) => {
            let hashed_passwords_match =
                validate_password(log_in.password.clone(), user.password).await;
            if !hashed_passwords_match {
                return HttpResponse::BadRequest().finish();
            }

            HttpResponse::Ok().json(LoginResponse {
                user_id: user.user_id.to_string(),
                token: generate_token(user.user_id.to_string()),
                expires_on: get_expires_at(Option::None),
            })
        }
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

#[tracing::instrument(
    name = "Reset password",
    skip(reset_password, pool, user_claim),
    fields(
        user_username = %reset_password.email_address,
    )
)]
pub async fn reset_password(
    reset_password: web::Json<ResetPassword>,
    pool: web::Data<PgPool>,
    user_claim: Claims,
) -> impl Responder {
    match get_user_by_email_address(&reset_password.email_address, &pool).await {
        Ok(user) => {
            if user_claim.user_id != user.user_id.to_string() {
                return HttpResponse::Unauthorized().finish();
            }

            let hashed_passwords_match =
                validate_password(reset_password.old_password.clone(), user.password).await;
            if !hashed_passwords_match {
                return HttpResponse::BadRequest().finish();
            }

            let new_hashed_password = hash_password(reset_password.new_password.clone()).await;

            match update_password(&reset_password.email_address, &new_hashed_password, &pool).await
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

#[tracing::instrument(
    name = "Forgot password",
    skip(forgot_password, pool),
    fields(
        user_username = %forgot_password.email_address,
    )
)]
pub async fn forgot_password(
    forgot_password: web::Json<ForgotPassword>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let email = standardize_email(&forgot_password.email_address);
    match get_user_by_email_address(email.as_str(), &pool).await {
        Ok(user) => {
            let otp = OneTimePasscode {
                id: Uuid::new_v4(),
                user_id: user.user_id.to_string(),
                one_time_passcode: Uuid::new_v4().to_string(),
                issued_on: Utc::now(),
                expires_on: Utc::now() + Duration::days(1),
                used: false,
            };
            match insert_otp(otp, &pool).await {
                Ok(_) => HttpResponse::Ok().json(json!({})),
                Err(err) => {
                    tracing::error!(
                        "Something happened while saving the one time passcode. {:?}",
                        err
                    );
                    HttpResponse::Ok().json(json!({}))
                }
            }
        }
        Err(_) => HttpResponse::Ok().json(json!({})),
    }
}
