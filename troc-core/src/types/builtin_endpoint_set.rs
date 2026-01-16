#![allow(clippy::identity_op)]
use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

use binrw::{BinRead, BinResult, BinWrite, Endian};
use modular_bitfield::{
    bitfield,
    prelude::{B3, B4, B16, B26},
    specifiers::{B1, B2, B5, B15},
};
use serde::{Deserialize, Serialize};

use super::is_target_little_endian;

#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    BinRead,
    BinWrite,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[br(import(_len: usize))]
pub struct BuiltinEndpointSet {
    #[br(parse_with = BuiltinEndpointSet::custom_parser)]
    #[bw(write_with = BuiltinEndpointSet::custom_writer)]
    set: BuiltinEndpointSetInner,
}

impl BuiltinEndpointSet {
    pub fn new() -> Self {
        Self {
            set: Default::default(),
        }
    }

    #[binrw::parser(reader, endian)]
    fn custom_parser() -> BinResult<BuiltinEndpointSetInner> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;

        if is_target_little_endian() && matches!(endian, Endian::Big) {
            buf.reverse();
        }

        let set = BuiltinEndpointSetInner::from_bytes(buf);
        Ok(set)
    }

    #[binrw::writer(writer, endian)]
    fn custom_writer(value: &BuiltinEndpointSetInner) -> BinResult<()> {
        let mut buf: [u8; 4] = BuiltinEndpointSetInner::into_bytes(*value);

        if is_target_little_endian() && matches!(endian, Endian::Big) {
            buf.reverse();
        }

        let _ = writer.write(&buf);
        Ok(())
    }
}

impl Deref for BuiltinEndpointSet {
    type Target = BuiltinEndpointSetInner;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl DerefMut for BuiltinEndpointSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

impl Display for BuiltinEndpointSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.set))?;
        Ok(())
    }
}

#[bitfield]
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuiltinEndpointSetInner {
    pub disc_builtin_endpoint_participant_announcer: B1,
    pub disc_builtin_endpoint_participant_detector: B1,
    pub disc_builtin_endpoint_publications_announcer: B1,
    pub disc_builtin_endpoint_publications_detector: B1,
    pub disc_builtin_endpoint_subscriptions_announcer: B1,
    pub disc_builtin_endpoint_subscriptions_detector: B1,
    pub unused_6: B4,
    pub builtin_endpoint_participant_message_data_writer: B1,
    pub builtin_endpoint_participant_message_data_reader: B1,
    pub unused_12: B16,
    pub disc_builtin_endpoint_topics_announcer: B1,
    pub disc_builtin_endpoint_topics_detector: B1,
    pub unused_30: B2,
}

impl Display for BuiltinEndpointSetInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tags = Vec::new();
        if self.disc_builtin_endpoint_participant_announcer() == 1 {
            tags.push("ParticipantAnnouncer");
        }
        if self.disc_builtin_endpoint_participant_detector() == 1 {
            tags.push("ParticipantDetector");
        }
        if self.disc_builtin_endpoint_publications_announcer() == 1 {
            tags.push("PublicationAnnouncer");
        }
        if self.disc_builtin_endpoint_publications_detector() == 1 {
            tags.push("PublicationDetector");
        }
        if self.disc_builtin_endpoint_subscriptions_announcer() == 1 {
            tags.push("SubscriptionAnnouncer");
        }
        if self.disc_builtin_endpoint_subscriptions_detector() == 1 {
            tags.push("SubscriptionDetector");
        }
        if self.builtin_endpoint_participant_message_data_writer() == 1 {
            tags.push("ParticipantMessageWriter");
        }
        if self.builtin_endpoint_participant_message_data_reader() == 1 {
            tags.push("ParticipantMessageReader");
        }
        if self.disc_builtin_endpoint_topics_announcer() == 1 {
            tags.push("TopicAnnouncer");
        }
        if self.disc_builtin_endpoint_topics_detector() == 1 {
            tags.push("TopicDetector");
        }

        f.write_str(&format!("{:?}", tags))?;
        Ok(())
    }
}
