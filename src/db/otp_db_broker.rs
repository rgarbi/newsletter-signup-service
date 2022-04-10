use sqlx::{Error, PgPool};

use crate::domain::otp_models::OneTimePasscode;

#[tracing::instrument(name = "Saving otp in the database", skip(one_time_passcode, pool))]
pub async fn insert_otp(one_time_passcode: OneTimePasscode, pool: &PgPool) -> Result<(), Error> {
    sqlx::query!(
        r#"INSERT 
            INTO otp (id, user_id, one_time_passcode, issued_on, expires_on, used) 
            VALUES ($1, $2, $3, $4, $5, $6)"#,
        one_time_passcode.id,
        one_time_passcode.user_id,
        one_time_passcode.one_time_passcode,
        one_time_passcode.issued_on,
        one_time_passcode.expires_on,
        one_time_passcode.used,
    )
    .execute(pool)
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Get otp by otp", skip(one_time_passcode, pool))]
pub async fn get_otp_by_otp(
    one_time_passcode: &str,
    pool: &PgPool,
) -> Result<OneTimePasscode, Error> {
    let result = sqlx::query!(
        r#"SELECT id, user_id, one_time_passcode, issued_on, expires_on, used
            FROM otp
            WHERE one_time_passcode = $1"#,
        one_time_passcode,
    )
    .fetch_one(pool)
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(OneTimePasscode {
        id: result.id,
        user_id: result.user_id,
        one_time_passcode: result.one_time_passcode,
        issued_on: result.issued_on,
        expires_on: result.expires_on,
        used: result.used,
    })
}

#[tracing::instrument(name = "Set to used by otp", skip(one_time_passcode, pool))]
pub async fn set_to_used_by_otp(one_time_passcode: &str, pool: &PgPool) -> Result<(), Error> {
    sqlx::query!(
        r#"UPDATE otp
            SET used = true
            WHERE one_time_passcode = $1"#,
        one_time_passcode,
    )
    .execute(pool)
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("{:?}", e);
        e
    })?;

    Ok(())
}
