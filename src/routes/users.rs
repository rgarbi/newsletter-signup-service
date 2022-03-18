use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::auth::password_hashing::{hash_password, validate_password};
use crate::auth::token::{generate_token, LoginResponse};
use crate::db::users::{count_users_with_username, get_user_by_username, insert_user};
use crate::domain::new_user::SignUp;

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

            let hashed_password = hash_password(sign_up.password.clone());
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
            let hashed_passwords_match = validate_password(sign_up.password.clone(), user.password);
            if !hashed_passwords_match {
                return HttpResponse::BadRequest().finish();
            }

            HttpResponse::Ok().json(LoginResponse {
                user_id: user.user_id.to_string().clone(),
                token: generate_token(user.user_id.to_string()),
            })
        }
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}
