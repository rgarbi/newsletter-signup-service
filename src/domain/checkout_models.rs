use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::log::error;
use uuid::Uuid;

use crate::domain::subscription_models::OverTheWireCreateSubscription;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateCheckoutSession {
    pub price_lookup_key: String,
    pub subscription: OverTheWireCreateSubscription,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateStripeSessionRedirect {
    pub location: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CheckoutSession {
    pub id: Uuid,
    pub user_id: String,
    pub session_state: CheckoutSessionState,
    pub created_at: DateTime<Utc>,
    pub price_lookup_key: String,
    pub subscription: serde_json::Value,
    pub stripe_session_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CheckoutSessionState {
    Created,
    CompletedSuccessfully,
    Cancelled,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StripeBillingPortalSession {
    pub id: String,
    pub object: String,
    pub configuration: String,
    pub created: u128,
    pub customer: String,
    pub livemode: bool,
    pub locale: Option<String>,
    pub on_behalf_of: Option<String>,
    pub return_url: String,
    pub url: String,
}

impl FromStr for CheckoutSessionState {
    type Err = ();

    fn from_str(val: &str) -> Result<CheckoutSessionState, ()> {
        if val.eq("Created") {
            return Ok(CheckoutSessionState::Created);
        }

        if val.eq("Cancelled") {
            return Ok(CheckoutSessionState::Cancelled);
        }

        if val.eq("CompletedSuccessfully") {
            return Ok(CheckoutSessionState::CompletedSuccessfully);
        }

        error!("Could not map string: {} to the enum HistoryEventType", val);
        Err(())
    }
}

impl CheckoutSessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckoutSessionState::Created => "Created",
            CheckoutSessionState::Cancelled => "Cancelled",
            CheckoutSessionState::CompletedSuccessfully => "CompletedSuccessfully",
        }
    }
}

impl CreateCheckoutSession {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl CheckoutSession {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl CreateStripeSessionRedirect {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::checkout_models::{
        CheckoutSession, CheckoutSessionState, CreateStripeSessionRedirect,
    };
    use chrono::Utc;
    use claim::{assert_err, assert_ok};
    use serde_json::json;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn checkout_session_state_from_str_works() {
        assert_ok!(CheckoutSessionState::from_str("Created"));
        assert_ok!(CheckoutSessionState::from_str("Cancelled"));
        assert_ok!(CheckoutSessionState::from_str("CompletedSuccessfully"));
        assert_err!(CheckoutSessionState::from_str(
            "not a part of the enum will blow up"
        ));
    }

    #[test]
    fn checkout_session_to_json_works() {
        let checkout_session = CheckoutSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4().to_string(),
            session_state: CheckoutSessionState::Created,
            created_at: Utc::now(),
            price_lookup_key: Uuid::new_v4().to_string(),
            subscription: json!("{}"),
            stripe_session_id: Uuid::new_v4().to_string(),
        };
        let _json_session = checkout_session.to_json();
    }

    #[test]
    fn create_stripe_session_redirect_to_json_works() {
        let redirect = CreateStripeSessionRedirect {
            location: Uuid::new_v4().to_string(),
        };
        let _redir = redirect.to_json();
    }
}
