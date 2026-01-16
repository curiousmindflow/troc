use crate::{ParticipantProxy, messages::Message, types::LocatorList};

use crate::discovery::{DiscoveredReaderData, DiscoveredWriterData};

#[derive(Debug)]
pub enum EffectConsumption {
    Consume,
    Left,
}

#[derive(Debug, Default)]
pub struct Effects(Vec<Effect>);

impl Effects {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn find(&self, predicate: impl Fn(&Effect) -> bool) -> Option<&Effect> {
        self.0.iter().find(|e| predicate(e))
    }

    pub fn push(&mut self, effect: Effect) {
        self.0.push(effect);
    }

    pub fn pop(&mut self) -> Option<Effect> {
        self.0.pop()
    }

    pub fn consume(&mut self, predicate: impl Fn(&Effect) -> EffectConsumption) {
        for (pos, e) in self.0.iter().enumerate() {
            if let EffectConsumption::Consume = predicate(e) {
                self.0.remove(pos);
                break;
            }
        }
    }

    pub fn clean(&mut self) {
        self.0.clear();
    }
}

#[derive(Debug)]
pub enum Effect {
    DataAvailable,
    Message {
        timestamp_millis: i64,
        message: Message,
        locators: LocatorList,
    },
    Qos, // should need additional fields also
    ParticipantMatch {
        participant_proxy: ParticipantProxy,
    },
    ParticipantRemoved {
        participant_proxy: ParticipantProxy,
    },
    ReaderMatch {
        success: bool,
        local_reader_infos: DiscoveredReaderData,
        remote_writer_infos: DiscoveredWriterData,
    },
    WriterMatch {
        success: bool,
        local_writer_infos: DiscoveredWriterData,
        remote_reader_infos: DiscoveredReaderData,
    },
    ScheduleTick {
        delay: i64,
    },
}
