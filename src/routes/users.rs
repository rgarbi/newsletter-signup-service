use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::auth::token::{generate_token, LoginResponse};
use crate::db::users::{count_users_with_username, insert_user};
use crate::domain::new_user::SignUp;

#[tracing::instrument(
    name = "Adding a new user",
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

            match insert_user(&sign_up, &pool).await {
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
