use sqlx::{Error, PgPool, Postgres, Transaction};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::user_models::{from_str_to_user_group, User, UserGroup};

#[tracing::instrument(name = "Count users with a given username", skip(email_address, pool))]
pub async fn count_users_with_email_address(
    email_address: &str,
    pool: &PgPool,
) -> Result<i64, Error> {
    let result = sqlx::query!(
        r#"SELECT COUNT(email_address) 
            FROM users 
            WHERE email_address = $1"#,
        email_address,
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

#[tracing::instrument(name = "Get user by email address", skip(email_address, pool))]
pub async fn get_user_by_email_address(email_address: &str, pool: &PgPool) -> Result<User, Error> {
    let result = sqlx::query!(
        r#"SELECT user_id, email_address, password, user_group
            FROM users 
            WHERE email_address = $1"#,
        email_address,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(User {
        user_id: result.user_id,
        email_address: result.email_address,
        password: result.password,
        user_group: from_str_to_user_group(result.user_group),
    })
}

#[tracing::instrument(name = "Get user by user_id", skip(user_id, pool))]
pub async fn get_user_by_user_id(user_id: &str, pool: &PgPool) -> Result<User, Error> {
    let id = Uuid::from_str(user_id).unwrap();
    let result = sqlx::query!(
        r#"SELECT user_id, email_address, password, user_group
            FROM users 
            WHERE user_id = $1"#,
        id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(User {
        user_id: result.user_id,
        email_address: result.email_address,
        password: result.password,
        user_group: from_str_to_user_group(result.user_group),
    })
}

#[tracing::instrument(
    name = "Saving new user in the database",
    skip(email_address, hashed_password, user_group, transaction)
)]
pub async fn insert_user(
    email_address: &str,
    hashed_password: &str,
    user_group: UserGroup,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<String, Error> {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT 
            INTO users (user_id, email_address, password, user_group) 
            VALUES ($1, $2, $3, $4)"#,
        user_id,
        email_address,
        hashed_password,
        user_group.as_str(),
    )
    .execute(transaction)
    .await
    .map_err(|e: Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(user_id.to_string())
}

#[tracing::instrument(
    name = "Update the password",
    skip(email_address, hashed_password, pool)
)]
pub async fn update_password(
    email_address: &str,
    hashed_password: &str,
    pool: &PgPool,
) -> Result<(), Error> {
    sqlx::query!(
        r#"UPDATE users
            SET password = $1
            WHERE email_address = $2"#,
        hashed_password,
        email_address,
    )
    .execute(pool)
    .await
    .map_err(|e: Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(())
}
