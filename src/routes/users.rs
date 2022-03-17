use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::auth::token::{generate_token, LoginResponse};
use crate::db::users::insert_user;
use crate::domain::new_user::SignUp;

#[tracing::instrument(
    name = "Adding a new user",
    skip(sign_up, pool),
    fields(
        user_username = %sign_up.username,
    )
)]
pub async fn sign_up(sign_up: web::Json<SignUp>, pool: web::Data<PgPool>) -> impl Responder {
    match insert_user(&sign_up, &pool).await {
        Ok(user_id) => HttpResponse::Ok().json(LoginResponse {
            user_id: user_id.clone(),
            token: generate_token(user_id),
        }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
