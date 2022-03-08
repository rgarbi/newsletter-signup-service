use serde::{Deserialize, Serialize};

use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;

#[derive(Deserialize, Serialize)]
pub struct OverTheWireCreateSubscriber {
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

impl OverTheWireCreateSubscriber {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

#[derive(Deserialize, Serialize)]
pub struct OverTheWireSubscriber {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

pub struct NewSubscriber {
    pub email_address: ValidEmail,
    pub first_name: ValidName,
    pub last_name: ValidName,
}
