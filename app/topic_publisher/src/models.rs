use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MomentoModel {
    pub key_one: String,
    pub key_two: String,
    pub key_three: i64,
    pub timestamp: DateTime<Utc>,
}

impl MomentoModel {
    pub fn new(key_one: String, key_two: String, key_three: i64) -> Self {
        MomentoModel {
            key_one,
            key_two,
            key_three,
            timestamp: Utc::now(),
        }
    }
}
