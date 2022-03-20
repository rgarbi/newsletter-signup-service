use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct SignUp {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct ResetPassword {
    pub username: String,
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
