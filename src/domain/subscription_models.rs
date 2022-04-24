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

impl NewSubscription {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl TryFrom<OverTheWireCreateSubscription> for NewSubscription {
    type Error = String;
    fn try_from(subscription: OverTheWireCreateSubscription) -> Result<Self, Self::Error> {
        let subscription_name = ValidName::parse(subscription.subscription_name)?;
        let subscription_email_address =
            ValidEmail::parse(subscription.subscription_email_address)?;
        Ok(NewSubscription {
            subscriber_id: subscription.subscriber_id,
            subscription_name,
            subscription_email_address,
            subscription_mailing_address_line_1: subscription.subscription_mailing_address_line_1,
            subscription_mailing_address_line_2: subscription.subscription_mailing_address_line_2,
            subscription_city: subscription.subscription_city,
            subscription_state: subscription.subscription_state,
            subscription_postal_code: subscription.subscription_postal_code,
            subscription_type: subscription.subscription_type,
            subscription_creation_date: Utc::now(),
            active: true,
        })
    }
}
