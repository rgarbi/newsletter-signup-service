use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub user_id: Uuid,
    pub email_address: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SignUp {
    pub email_address: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LogIn {
    pub email_address: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct ResetPassword {
    pub email_address: String,
    pub old_password: String,
    pub new_password: String,
}

impl SignUp {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl ResetPassword {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl LogIn {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}
