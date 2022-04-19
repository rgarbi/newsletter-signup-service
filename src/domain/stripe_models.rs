use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateCheckoutSession {
    pub price_lookup_key: String,
}

impl CreateCheckoutSession {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

#[derive(Deserialize, Serialize)]
pub enum StripeWebhookEventType {
    Created,
    ChangedPaymentMethod,
    Cancelled,
    UpdatedSubscriptionInformation,
}
