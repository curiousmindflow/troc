use serde::{Deserialize, Serialize};

use super::Timestamp;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct TimeBasedFilterQosPolicy {
    pub minimum_separation: Timestamp,
}
