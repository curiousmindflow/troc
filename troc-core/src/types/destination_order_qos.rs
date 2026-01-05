use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DestinationOrderQosPolicy {
    #[default]
    BySourceTimestamp,
    ByReceptionTimestamp,
}
