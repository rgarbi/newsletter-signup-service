use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::subscription_models::OverTheWireCreateSubscription;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateCheckoutSession {
    pub price_lookup_key: String,
    pub subscription: OverTheWireCreateSubscription,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CheckoutSession {
    pub user_id: String,
    pub session_state: CheckoutSessionState,
    pub created_at: DateTime<Utc>,
    pub price_lookup_key: String,
    pub subscription: OverTheWireCreateSubscription,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CheckoutSessionState {
    Created,
    CompletedSuccessfully,
    Cancelled,
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
