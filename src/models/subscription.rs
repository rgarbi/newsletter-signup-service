use std::time::SystemTime;

use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
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
#[serde(crate = "rocket::serde")]
enum SubscriptionType {
    Electronic,
    Physical,
}

impl Subscription {
    pub fn new_subscription() -> Subscription {
        Subscription {
            id: Option::from(1234),
            subscriber_id: 1,
            subscription_first_name: String::from("first name"),
            subscription_last_name: String::from(" last name"),
            subscription_mailing_address_line_1: String::from("address line 1"),
            subscription_mailing_address_line_2: String::from("address line 2"),
            subscription_city: String::from("Waco"),
            subscription_state: String::from("Texas"),
            subscription_postal_code: String::from("76633"),
            subscription_email_address: String::from("test.test@emailaddress.com"),
            subscription_creation_date: DateTime::from(SystemTime::now()),
            active: true,
            subscription_type: SubscriptionType::Electronic,
        }
    }
}
