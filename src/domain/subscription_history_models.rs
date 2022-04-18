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

#[derive(Deserialize, Serialize)]
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

    pub fn from_str(val: String) -> HistoryEventType {
        if val.eq("Created") {
            return HistoryEventType::Created;
        }

        if val.eq("Cancelled") {
            return HistoryEventType::Cancelled;
        }

        if val.eq("ChangedPaymentMethod") {
            return HistoryEventType::ChangedPaymentMethod;
        }

        if val.eq("UpdatedSubscriptionInformation") {
            return HistoryEventType::UpdatedSubscriptionInformation;
        }

        error!("Could not map string: {} to the enum HistoryEventType", val);
        HistoryEventType::UpdatedSubscriptionInformation
    }
}
