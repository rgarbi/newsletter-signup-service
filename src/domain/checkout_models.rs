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
            CheckoutSessionState::CompletedSuccessfully => "ChangedPaymentMethod",
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