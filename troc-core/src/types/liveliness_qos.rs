use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

use super::Timestamp;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[brw(repr = u32)]
#[br(import(_len: usize))]
pub enum LivelinessKind {
    #[default]
    Automatic,
    ManualByParticipant,
    ManualByTopic,
}

impl Display for LivelinessKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LivelinessKind::Automatic => f.write_str("LivelinessKind::Automatic ")?,
            LivelinessKind::ManualByParticipant => {
                f.write_str("LivelinessKind::ManualByParticipant")?
            }
            LivelinessKind::ManualByTopic => f.write_str("LivelinessKind::ManualByTopic")?,
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[br(import(_len: usize))]
pub struct LivelinessQosPolicy {
    pub kind: LivelinessKind,
    pub lease_duration: Timestamp,
}

impl Display for LivelinessQosPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Kind: {}, lease_duration: {}",
            self.kind, self.lease_duration
        ))?;
        Ok(())
    }
}
