use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Subscription {
    id: String,
    subscriber_id: String,
    subscription_first_name: String,
    subscription_last_name: String,
    subscription_mailing_address_line_1: String,
    subscription_mailing_address_line_2: Option<String>,
    subscription_city: String,
    subscription_state: String,
    subscription_postal_code: String,
    subscription_email_address: String,
    subscription_creation_date: Option<DateTime<Utc>>,
    active: Option<bool>,
    subscription_type: SubscriptionType,
}

#[derive(Deserialize, Serialize)]
enum SubscriptionType {
    Electronic,
    Physical,
}

impl SubscriptionType {
    fn as_str(&self) -> &'static str {
        match self {
            SubscriptionType::Electronic => "Electronic",
            SubscriptionType::Physical => "Physical",
        }
    }
}

#[tracing::instrument(
name = "Adding a new subscriber",
skip(subscription, pool),
fields(
request_id = %Uuid::new_v4(),
subscriber_id = %subscription.subscriber_id,
subscription_email_address = %subscription.subscription_email_address,
)
)]
pub async fn post_subscription(
    subscription: web::Form<Subscription>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    match insert_subscription(&subscription, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscription details in the database",
    skip(subscription, pool)
)]
pub async fn insert_subscription(
    subscription: &Subscription,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions (
            id, 
            subscriber_id, 
            subscription_first_name, 
            subscription_last_name, 
            subscription_mailing_address_line_1, 
            subscription_mailing_address_line_2,
            subscription_city,
            subscription_state,
            subscription_postal_code,
            subscription_email_address,
            subscription_creation_date,
            active,
            subscription_type
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
        Uuid::from_str(&*subscription.id).expect("Unable to parse the UUID"),
        Uuid::from_str(&*subscription.subscriber_id).expect("Unable to parse the UUID"),
        subscription.subscription_first_name,
        subscription.subscription_last_name,
        subscription.subscription_mailing_address_line_1,
        subscription.subscription_mailing_address_line_2,
        subscription.subscription_city,
        subscription.subscription_state,
        subscription.subscription_postal_code,
        subscription.subscription_email_address,
        Utc::now(),
        true,
        subscription.subscription_type.as_str()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
