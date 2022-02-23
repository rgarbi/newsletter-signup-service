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

pub async fn subscription(
    subscription: web::Form<Subscription>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    tracing::info!(
        "request id {} - Adding a new subscription for the subscriber {} with a subscription type of {}",
        request_id,
        subscription.subscriber_id,
        subscription.subscription_type.as_str()
    );
    let result = sqlx::query!(
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
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            tracing::info!(
                "request id {} - New subscription details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                "request id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
