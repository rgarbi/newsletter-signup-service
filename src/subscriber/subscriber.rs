use rocket::serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Subscriber {
    id: Option<i32>,
    first_name: String,
    last_name: DateTime<Utc>,
    email_address: String,
}