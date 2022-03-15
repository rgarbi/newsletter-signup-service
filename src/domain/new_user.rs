use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
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
