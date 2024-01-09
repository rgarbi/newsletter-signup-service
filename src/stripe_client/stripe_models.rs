use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StripeBillingPortalSession {
    pub id: String,
    pub object: String,
    pub configuration: String,
    pub created: u64,
    pub customer: String,
    pub livemode: bool,
    pub locale: Option<String>,
    pub on_behalf_of: Option<String>,
    pub return_url: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StripeSessionObject {
    pub id: String,
    pub object: String,
    pub amount_subtotal: u64,
    pub amount_total: u64,
    pub client_reference_id: Option<String>,
    pub customer: String,
    pub subscription: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StripeCustomer {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub description: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StripeProductPrice {
    pub id: String,
    pub object: String,
    pub active: bool,
    pub billing_scheme: String,
    pub created: u64,
    pub currency: String,
    pub product: String,
    pub unit_amount: u64,
    pub lookup_key: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StripePriceList {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<StripeProductPrice>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StripeCheckoutSession {
    pub id: String,
    pub object: String,
    pub cancel_url: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StripeWebhookEvent {
    pub id: String,
    pub data: String,
    pub created: u64,
    #[serde(alias = "type")]
    pub event_type: String
}

impl StripeSessionObject {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl StripeCustomer {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

#[cfg(test)]
mod tests {
    use crate::stripe_client::stripe_models::{StripeCustomer, StripeSessionObject};
    use uuid::Uuid;

    #[test]
    fn stripe_customer_to_json_works() {
        let stripe_customer = StripeCustomer {
            id: Uuid::new_v4().to_string(),
            object: Uuid::new_v4().to_string(),
            created: 1233222,
            description: None,
            email: None,
        };
        let _json = stripe_customer.to_json();
    }

    #[test]
    fn stripe_session_object_to_json_works() {
        let stripe_session_object = StripeSessionObject {
            id: Uuid::new_v4().to_string(),
            object: Uuid::new_v4().to_string(),
            amount_subtotal: 0,
            amount_total: 0,
            client_reference_id: None,
            customer: "".to_string(),
            subscription: None,
        };
        let _json = stripe_session_object.to_json();
    }
}
