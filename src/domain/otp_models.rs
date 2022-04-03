use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct OneTimePasscode {
    pub id: Uuid,
    pub user_id: String,
    pub one_time_passcode: String,
    pub issued_on: DateTime<Utc>,
    pub expires_on: DateTime<Utc>,
    pub used: bool,
}
