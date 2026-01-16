use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

use super::duration::Duration;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[br(import(_len: usize))]
#[br(map = HistoryQosPolicy::map_read)]
#[bw(map = HistoryQosPolicy::map_write)]
pub enum HistoryQosPolicy {
    KeepAll,
    KeepLast { depth: u32 },
}

#[binrw]
struct WireHistoryQosPolicy {
    kind: u32,
    depth: u32,
}

impl HistoryQosPolicy {
    fn map_read(value: WireHistoryQosPolicy) -> HistoryQosPolicy {
        match value.kind {
            0 => HistoryQosPolicy::KeepLast { depth: value.depth },
            1 => HistoryQosPolicy::KeepAll,
            _ => unreachable!(),
        }
    }

    fn map_write(value: &HistoryQosPolicy) -> WireHistoryQosPolicy {
        match value {
            HistoryQosPolicy::KeepLast { depth } => WireHistoryQosPolicy {
                kind: 0,
                depth: *depth,
            },
            HistoryQosPolicy::KeepAll => WireHistoryQosPolicy { kind: 1, depth: 0 },
        }
    }
}

impl Default for HistoryQosPolicy {
    fn default() -> Self {
        Self::KeepLast { depth: 1 }
    }
}

impl Display for HistoryQosPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryQosPolicy::KeepAll => f.write_str("HistoryQosPolicy::KeepAll")?,
            HistoryQosPolicy::KeepLast { depth } => f.write_str(&format!(
                "HistoryQosPolicy::KeepLast {{ depth: {} }}",
                depth
            ))?,
        }
        Ok(())
    }
}
