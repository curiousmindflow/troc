use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[brw(repr = u32)]
#[br(import(_len: usize))]
pub enum DurabilityQosPolicy {
    #[default]
    Volatile,
    TransientLocal,
}

impl Display for DurabilityQosPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DurabilityQosPolicy::Volatile => f.write_str("DurabilityQosPolicy::Volatile")?,
            DurabilityQosPolicy::TransientLocal => {
                f.write_str("DurabilityQosPolicy::TransientLocal")?
            }
        }
        Ok(())
    }
}
