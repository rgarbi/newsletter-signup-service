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
