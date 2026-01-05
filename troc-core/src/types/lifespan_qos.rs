use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

use super::Timestamp;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[br(import(_len: usize))]
pub struct LifespanQosPolicy {
    pub duration: Timestamp,
}

impl Display for LifespanQosPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "LifespanQosPolicy {{ duration: {} }}",
            self.duration
        ))?;
        Ok(())
    }
}
