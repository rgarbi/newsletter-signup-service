use serde::{Deserialize, Serialize};

use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

#[derive(Deserialize, Serialize)]
pub struct OverTheWireCreateSubscriber {
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

#[derive(Deserialize, Serialize)]
pub struct OverTheWireSubscriber {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

pub struct NewSubscriber {
    pub email_address: SubscriberEmail,
    pub first_name: SubscriberName,
    pub last_name: SubscriberName,
}
