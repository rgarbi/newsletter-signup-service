use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub event_text: String,
    pub sent_on: DateTime<Utc>,
    pub processed: bool,
}

impl WebhookEvent {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}
