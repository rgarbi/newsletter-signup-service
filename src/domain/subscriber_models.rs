use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;

#[derive(Deserialize, Serialize)]
pub struct OverTheWireCreateSubscriber {
    pub name: String,
    pub email_address: String,
    pub user_id: String,
}

impl OverTheWireCreateSubscriber {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

#[derive(Deserialize, Serialize)]
pub struct OverTheWireSubscriber {
    pub id: Uuid,
    pub name: String,
    pub email_address: String,
    pub user_id: String,
    pub stripe_customer_id: Option<String>,
}

pub struct NewSubscriber {
    pub email_address: ValidEmail,
    pub name: ValidName,
    pub user_id: String,
}
