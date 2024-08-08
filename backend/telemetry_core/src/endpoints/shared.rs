use chrono::DateTime;
use serde::Serialize;

use crate::state::UniqueNodeIdentity;

// S as in Serialized
#[derive(Debug, Clone, Serialize)]
pub struct SUniqueNodeIdentity {
    pub node_name: Box<str>,
    pub network_id: Box<str>,
}

impl From<&UniqueNodeIdentity> for SUniqueNodeIdentity {
    fn from(value: &UniqueNodeIdentity) -> Self {
        Self {
            node_name: value.node_name.as_ref().into(),
            network_id: value.network_id.as_ref().into(),
        }
    }
}

impl From<UniqueNodeIdentity> for SUniqueNodeIdentity {
    fn from(value: UniqueNodeIdentity) -> Self {
        Self {
            node_name: value.node_name.as_ref().into(),
            network_id: value.network_id.as_ref().into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SDateTime {
    timestamp: i64,
    date: Box<str>,
}

impl From<u64> for SDateTime {
    fn from(value: u64) -> Self {
        let date_time = DateTime::from_timestamp_millis(value as i64).unwrap_or_default();
        Self {
            timestamp: date_time.timestamp_millis(),
            date: date_time.to_string().into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BlockProducer {
    pub identity: SUniqueNodeIdentity,
    pub start: SDateTime,
    pub end: SDateTime,
}
