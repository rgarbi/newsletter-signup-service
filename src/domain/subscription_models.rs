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
    pub subscription_cancelled_on_date: Option<DateTime<Utc>>,
    pub subscription_anniversary_day: i32,
    pub subscription_anniversary_month: i32,
    pub active: bool,
    pub subscription_type: SubscriptionType,
    pub stripe_subscription_id: String,
}

#[derive(Deserialize, Serialize, Clone)]
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
    pub subscription_anniversary_day: i32,
    pub subscription_anniversary_month: i32,
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

impl OverTheWireSubscription {
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
            subscription_anniversary_day: 0,
            subscription_anniversary_month: 0,
            active: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::subscription_models::{
        NewSubscription, OverTheWireCreateSubscription, OverTheWireSubscription, SubscriptionType,
    };
    use crate::domain::valid_email::ValidEmail;
    use crate::domain::valid_name::ValidName;
    use chrono::Utc;

    #[test]
    fn subscription_type_from_string_test() {
        assert_eq!("Digital", SubscriptionType::Digital.as_str());
        assert_eq!("Paper", SubscriptionType::Paper.as_str());
    }

    #[test]
    fn over_the_wire_create_subscription_to_json_works() {
        let over_the_wire_create_subscription = OverTheWireCreateSubscription {
            subscriber_id: "".to_string(),
            subscription_name: "".to_string(),
            subscription_mailing_address_line_1: "".to_string(),
            subscription_mailing_address_line_2: None,
            subscription_city: "".to_string(),
            subscription_state: "".to_string(),
            subscription_postal_code: "".to_string(),
            subscription_email_address: "".to_string(),
            subscription_type: SubscriptionType::Digital,
        };
        let _json = over_the_wire_create_subscription.to_json();
    }

    #[test]
    fn new_subscription_to_json_works() {
        let new_subscription = NewSubscription {
            subscriber_id: "".to_string(),
            subscription_name: ValidName::parse("A Name".to_string()).unwrap(),
            subscription_mailing_address_line_1: "".to_string(),
            subscription_mailing_address_line_2: None,
            subscription_city: "".to_string(),
            subscription_state: "".to_string(),
            subscription_postal_code: "".to_string(),
            subscription_email_address: ValidEmail::parse("someone@gmail.com".to_string()).unwrap(),
            subscription_creation_date: Utc::now(),
            subscription_anniversary_day: 0,
            subscription_anniversary_month: 0,
            active: false,
            subscription_type: SubscriptionType::Paper,
        };
        let _json = new_subscription.to_json();
    }

    #[test]
    fn over_the_wire_subscription_to_json_works() {
        let over_the_wire_subscription = OverTheWireSubscription {
            id: Default::default(),
            subscriber_id: Default::default(),
            subscription_name: "".to_string(),
            subscription_mailing_address_line_1: "".to_string(),
            subscription_mailing_address_line_2: "".to_string(),
            subscription_city: "".to_string(),
            subscription_state: "".to_string(),
            subscription_postal_code: "".to_string(),
            subscription_email_address: "".to_string(),
            subscription_creation_date: Utc::now(),
            subscription_cancelled_on_date: None,
            subscription_anniversary_day: 1,
            subscription_anniversary_month: 12,
            active: false,
            subscription_type: SubscriptionType::Digital,
            stripe_subscription_id: "".to_string(),
        };
        let _json = over_the_wire_subscription.to_json();
    }
}
