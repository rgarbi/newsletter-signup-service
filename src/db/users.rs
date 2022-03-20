use actix_web::ResponseError;
use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::domain::new_user::User;

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

#[tracing::instrument(name = "Get user by username", skip(username, pool))]
pub async fn get_user_by_username(username: &str, pool: &PgPool) -> Result<User, Error> {
    let result = sqlx::query!(
        r#"SELECT user_id, username, password, salt
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

    Ok(User {
        user_id: result.user_id,
        username: result.username,
        password: result.password,
    })
}

#[tracing::instrument(
    name = "Saving new user in the database",
    skip(username, hashed_password, pool)
)]
pub async fn insert_user(
    username: &str,
    hashed_password: &str,
    pool: &PgPool,
) -> Result<String, Error> {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT 
            INTO users (user_id, username, password) 
            VALUES ($1, $2, $3)"#,
        user_id,
        username,
        hashed_password,
    )
    .execute(pool)
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(user_id.to_string())
}

#[tracing::instrument(name = "Update the password", skip(username, hashed_password, pool))]
pub async fn update_password(
    username: &str,
    hashed_password: &str,
    pool: &PgPool,
) -> Result<(), Error> {
    sqlx::query!(
        r#"UPDATE users
            SET 
                password = $1
            WHERE username = $2"#,
        hashed_password,
        username,
    )
    .execute(pool)
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(())
}

#[derive(Debug)]
pub struct UserDatabaseError(sqlx::Error);

impl ResponseError for UserDatabaseError {}

impl std::fmt::Display for UserDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\nCaused by:\n\t{:?}", self, self.0)
    }
}
