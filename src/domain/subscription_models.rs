use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OverTheWireCreateSubscription {
    pub subscriber_id: String,
    pub subscription_name: String,
    pub subscription_mailing_address_line_1: String,
    pub subscription_mailing_address_line_2: Option<String>,
    pub subscription_city: String,
    pub subscription_state: String,
    pub subscription_postal_code: String,
    pub subscription_email_address: String,
    pub subscription_type: SubscriptionType,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct OverTheWireSubscription {
    pub id: Uuid,
    pub subscriber_id: Uuid,
    pub subscription_name: String,
    pub subscription_mailing_address_line_1: String,
    pub subscription_mailing_address_line_2: String,
    pub subscription_city: String,
    pub subscription_state: String,
    pub subscription_postal_code: String,
    pub subscription_email_address: String,
    pub subscription_creation_date: DateTime<Utc>,
    pub active: bool,
    pub subscription_type: SubscriptionType,
    pub stripe_subscription_id: String,
}

pub struct NewSubscription {
    pub subscriber_id: String,
    pub subscription_name: ValidName,
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SubscriptionType {
    Digital,
    Paper,
}

impl SubscriptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionType::Digital => "Digital",
            SubscriptionType::Paper => "Paper",
        }
    }
}

impl OverTheWireCreateSubscription {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}