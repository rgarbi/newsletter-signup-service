use rocket::serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Subscriber {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<i32>,
    first_name: String,
    last_name: String,
    email_address: String,
}

impl Subscriber {
    pub fn new_subscriber(id: Option<i32>, first_name: String, last_name: String, email_address: String) -> Subscriber {
        Subscriber {
            id,
            first_name,
            last_name,
            email_address,
        }
    }

    pub async fn get_subscriber_by_id(id: i32) -> Option<Subscriber> {
        Option::from(Subscriber::new_subscriber(Option::from(id), String::from("first"), String::from("last"), String::from("first.last@somemail.com")))
    }
}