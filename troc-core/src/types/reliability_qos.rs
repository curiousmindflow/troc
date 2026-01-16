use std::fmt::Display;

use binrw::{Endian, binrw};
use serde::{Deserialize, Serialize};

use crate::types::ReliabilityKind;

use super::{Timestamp, duration::Duration};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[br(import(_len: usize))]
#[br(map = ReliabilityQosPolicy::map_read)]
#[bw(map = ReliabilityQosPolicy::map_write)]
pub enum ReliabilityQosPolicy {
    Reliable { max_blocking_time: Timestamp },
    BestEffort,
}

#[binrw]
struct WireReliabilityQosPolicy {
    kind: u32,
    max_blocking_time: Timestamp,
}

impl ReliabilityQosPolicy {
    fn map_read(value: WireReliabilityQosPolicy) -> ReliabilityQosPolicy {
        match value.kind {
            1 => ReliabilityQosPolicy::BestEffort,
            2 => ReliabilityQosPolicy::Reliable {
                max_blocking_time: Default::default(),
            },
            _ => unreachable!(),
        }
    }

    fn map_write(value: &ReliabilityQosPolicy) -> WireReliabilityQosPolicy {
        match value {
            ReliabilityQosPolicy::BestEffort => WireReliabilityQosPolicy {
                kind: 1,
                max_blocking_time: Timestamp::default(),
            },
            ReliabilityQosPolicy::Reliable { max_blocking_time } => WireReliabilityQosPolicy {
                kind: 2,
                max_blocking_time: *max_blocking_time,
            },
        }
    }
}

impl Default for ReliabilityQosPolicy {
    fn default() -> Self {
        Self::BestEffort
    }
}

impl From<ReliabilityQosPolicy> for ReliabilityKind {
    fn from(value: ReliabilityQosPolicy) -> Self {
        match value {
            ReliabilityQosPolicy::Reliable {
                max_blocking_time: _,
            } => ReliabilityKind::Reliable,
            ReliabilityQosPolicy::BestEffort => ReliabilityKind::BestEffort,
        }
    }
}

impl Display for ReliabilityQosPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReliabilityQosPolicy::BestEffort => f.write_str("ReliabilityQosPolicy::BestEffort")?,
            ReliabilityQosPolicy::Reliable { max_blocking_time } => f.write_str(&format!(
                "ReliabilityQosPolicy::Reliable {{ max_blocking_time: {} }}",
                max_blocking_time
            ))?,
        }
        Ok(())
    }
}
