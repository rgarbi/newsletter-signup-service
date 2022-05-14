use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::log::error;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct SubscriptionHistoryEvent {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub subscription_change_event_date: DateTime<Utc>,
    pub subscription_change_event_type: HistoryEventType,
    pub subscription: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum HistoryEventType {
    Created,
    ChangedPaymentMethod,
    Cancelled,
    UpdatedSubscriptionInformation,
}

impl HistoryEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HistoryEventType::Created => "Created",
            HistoryEventType::Cancelled => "Cancelled",
            HistoryEventType::ChangedPaymentMethod => "ChangedPaymentMethod",
            HistoryEventType::UpdatedSubscriptionInformation => "UpdatedSubscriptionInformation",
        }
    }
}

impl FromStr for HistoryEventType {
    type Err = ();

    fn from_str(val: &str) -> Result<HistoryEventType, ()> {
        if val.eq("Created") {
            return Ok(HistoryEventType::Created);
        }

        if val.eq("Cancelled") {
            return Ok(HistoryEventType::Cancelled);
        }

        if val.eq("ChangedPaymentMethod") {
            return Ok(HistoryEventType::ChangedPaymentMethod);
        }

        if val.eq("UpdatedSubscriptionInformation") {
            return Ok(HistoryEventType::UpdatedSubscriptionInformation);
        }

        error!("Could not map string: {} to the enum HistoryEventType", val);
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::subscription_history_models::HistoryEventType;
    use claim::{assert_err, assert_ok};
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn history_event_type_from_string_test() {
        assert_ok!(HistoryEventType::from_str("Created"));
        assert_ok!(HistoryEventType::from_str("Cancelled"));
        assert_ok!(HistoryEventType::from_str("ChangedPaymentMethod"));
        assert_ok!(HistoryEventType::from_str("UpdatedSubscriptionInformation"));
        assert_err!(HistoryEventType::from_str(
            Uuid::new_v4().to_string().as_str()
        ));
    }
}
