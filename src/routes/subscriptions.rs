use actix_web::{HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Subscription {
    id: Option<i64>,
    subscriber_id: i64,
    subscription_first_name: String,
    subscription_last_name: String,
    subscription_mailing_address_line_1: String,
    subscription_mailing_address_line_2: String,
    subscription_city: String,
    subscription_state: String,
    subscription_postal_code: String,
    subscription_email_address: String,
    subscription_creation_date: DateTime<Utc>,
    active: bool,
    subscription_type: SubscriptionType,
}

#[derive(Deserialize, Serialize)]
enum SubscriptionType {
    Electronic,
    Physical,
}

pub async fn subscription() -> impl Responder {
    HttpResponse::Ok()
}
