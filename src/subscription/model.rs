use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Subscription {
    id: Option<i32>,
    subscriber_id: i32,
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
#[serde(crate = "rocket::serde")]
enum SubscriptionType {
    Electronic,
    Physical,
}
