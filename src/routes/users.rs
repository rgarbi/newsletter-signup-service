use actix_web::{web, HttpResponse, Responder, ResponseError};
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

#[tracing::instrument(name = "Count users with a given username", skip(username, pool))]
pub async fn count_users_with_username(username: &str, pool: &PgPool) -> Result<i64, Error> {
    let result = sqlx::query!(
        r#"SELECT COUNT(username) 
            FROM users 
            WHERE username = $1"#,
        username,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("{:?}", e);
        e
    })?;

    let count = result.count.unwrap_or_default();
    Ok(count)
}

#[tracing::instrument(name = "Saving new user in the database", skip(sign_up, pool))]
pub async fn insert_user(sign_up: &SignUp, pool: &PgPool) -> Result<String, Error> {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT 
            INTO users (user_id, username, password) 
            VALUES ($1, $2, $3)"#,
        user_id,
        sign_up.username,
        sign_up.password,
    )
    .execute(pool)
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(user_id.to_string())
}

#[derive(Debug)]
pub struct UserDatabaseError(sqlx::Error);

impl ResponseError for UserDatabaseError {}

impl std::fmt::Display for UserDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\nCaused by:\n\t{:?}", self, self.0)
    }
}
