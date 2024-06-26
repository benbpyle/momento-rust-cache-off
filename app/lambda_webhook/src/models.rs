use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Main Payload coming from the Momento Webhook
#[derive(Serialize, Deserialize, Debug)]
pub struct MomentoPayload {
    pub cache: String,
    pub topic: String,
    pub event_timestamp: i64,
    pub publish_timestamp: i64,
    pub topic_sequence_number: i64,
    pub token_id: Option<String>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MomentoModel {
    pub key_one: String,
    pub key_two: String,
    pub key_three: i64,
    #[serde()]
    #[serde(rename(deserialize = "timestamp"))]
    pub published_timestamp: DateTime<Utc>,
    #[serde(skip)]
    pub received_timestamp: DateTime<Utc>,
}

impl MomentoModel {
    pub fn time_between_mo_publish_and_received(&self, mo_timestamp: i64) -> i64 {
        let mo_time = DateTime::from_timestamp_millis(mo_timestamp);
        self.received_timestamp
            .signed_duration_since(mo_time.unwrap())
            .num_milliseconds()
    }

    pub fn time_between_publish_and_received(&self) -> i64 {
        self.received_timestamp
            .signed_duration_since(self.published_timestamp)
            .num_milliseconds()
    }
}
