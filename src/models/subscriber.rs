use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Subscriber {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

impl Subscriber {
    pub fn new_subscriber(
        id: Option<i64>,
        first_name: String,
        last_name: String,
        email_address: String,
    ) -> Subscriber {
        Subscriber {
            id,
            first_name,
            last_name,
            email_address,
        }
    }
}
