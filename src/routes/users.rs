use actix_web::{web, HttpResponse, Responder};
use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::auth::token::{generate_token, LoginResponse};
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

#[tracing::instrument(name = "Saving new user in the database", skip(sign_up, pool))]
pub async fn insert_user(sign_up: &SignUp, pool: &PgPool) -> Result<String, Error> {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT 
            INTO users (user_id, username, password) 
            VALUES ($1, $2, $3) 
            ON CONFLICT (username) DO NOTHING"#,
        user_id,
        sign_up.username,
        sign_up.password,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(user_id.to_string())
}
