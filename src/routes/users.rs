use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::auth::password_hashing::{hash_password, validate_password};
use crate::auth::token::{generate_token, Claims, LoginResponse};
use crate::db::users::{
    count_users_with_username, get_user_by_username, insert_user, update_password,
};
use crate::domain::new_user::{ResetPassword, SignUp};

#[tracing::instrument(
    name = "Singing up a new user",
    skip(sign_up, pool),
    fields(
        user_username = %sign_up.username,
    )
)]
pub async fn sign_up(sign_up: web::Json<SignUp>, pool: web::Data<PgPool>) -> impl Responder {
    match count_users_with_username(&sign_up.username, &pool).await {
        Ok(count) => {
            if count > 0 {
                return HttpResponse::Conflict().finish();
            }

            let hashed_password = hash_password(sign_up.password.clone()).await;
            match insert_user(&sign_up.username, &hashed_password, &pool).await {
                Ok(user_id) => HttpResponse::Ok().json(LoginResponse {
                    user_id: user_id.clone(),
                    token: generate_token(user_id),
                }),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
name = "Login user",
skip(sign_up, pool),
fields(
user_username = %sign_up.username,
)
)]
pub async fn login(sign_up: web::Json<SignUp>, pool: web::Data<PgPool>) -> impl Responder {
    match get_user_by_username(&sign_up.username, &pool).await {
        Ok(user) => {
            let hashed_passwords_match =
                validate_password(sign_up.password.clone(), user.password).await;
            if !hashed_passwords_match {
                return HttpResponse::BadRequest().finish();
            }

            HttpResponse::Ok().json(LoginResponse {
                user_id: user.user_id.to_string(),
                token: generate_token(user.user_id.to_string()),
            })
        }
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

#[tracing::instrument(
name = "Reset password",
skip(reset_password, pool, user_claim),
fields(
user_username = %reset_password.username,
)
)]
pub async fn reset_password(
    reset_password: web::Json<ResetPassword>,
    pool: web::Data<PgPool>,
    user_claim: Claims,
) -> impl Responder {
    match get_user_by_username(&reset_password.username, &pool).await {
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

            match update_password(&reset_password.username, &new_hashed_password, &pool).await {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}
