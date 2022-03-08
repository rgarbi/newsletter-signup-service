use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct OverTheWireCreateSubscription {
    pub subscriber_id: String,
    pub subscription_first_name: String,
    pub subscription_last_name: String,
    pub subscription_mailing_address_line_1: String,
    pub subscription_mailing_address_line_2: Option<String>,
    pub subscription_city: String,
    pub subscription_state: String,
    pub subscription_postal_code: String,
    pub subscription_email_address: String,
    pub subscription_type: SubscriptionType,
}

#[derive(Deserialize, Serialize)]
pub struct OverTheWireSubscription {
    pub id: String,
    pub subscriber_id: String,
    pub subscription_first_name: String,
    pub subscription_last_name: String,
    pub subscription_mailing_address_line_1: String,
    pub subscription_mailing_address_line_2: Option<String>,
    pub subscription_city: String,
    pub subscription_state: String,
    pub subscription_postal_code: String,
    pub subscription_email_address: String,
    pub subscription_creation_date: Option<DateTime<Utc>>,
    pub active: bool,
    pub subscription_type: SubscriptionType,
}

pub struct NewSubscription {
    pub subscriber_id: String,
    pub subscription_first_name: ValidName,
    pub subscription_last_name: ValidName,
    pub subscription_mailing_address_line_1: String,
    pub subscription_mailing_address_line_2: Option<String>,
    pub subscription_city: String,
    pub subscription_state: String,
    pub subscription_postal_code: String,
    pub subscription_email_address: ValidEmail,
    pub subscription_creation_date: DateTime<Utc>,
    pub active: bool,
    pub subscription_type: SubscriptionType,
}

#[derive(Deserialize, Serialize)]
pub enum SubscriptionType {
    Electronic,
    Physical,
}

impl SubscriptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionType::Electronic => "Electronic",
            SubscriptionType::Physical => "Physical",
        }
    }
}

impl OverTheWireCreateSubscription {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}
