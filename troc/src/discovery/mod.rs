use troc_core::{EntityId, GuidPrefix, ParticipantProxy, ReaderProxy, WriterProxy};

mod disc;

pub use disc::{DiscoveryActor, DiscoveryActorCreateObject, DiscoveryActorMessage};

#[derive(Debug)]
pub enum DiscoveryEvent {
    WriterDetected {
        writer_id: EntityId,
        proxy: WriterProxy,
    },
    WriterMatched {
        writer_id: EntityId,
        proxy: WriterProxy,
    },
    WriterRemoved {
        writer_id: EntityId,
    },
    ReaderDetected {
        reader_id: EntityId,
        proxy: ReaderProxy,
    },
    ReaderMatched {
        reader_id: EntityId,
        proxy: ReaderProxy,
    },
    ReaderRemoved {
        reader_id: EntityId,
    },
    ParticipantDiscovered {
        proxy: ParticipantProxy,
    },
    ParticipantUpdated {
        proxy: ParticipantProxy,
    },
    ParticipantRemoved {
        guid_prefix: GuidPrefix,
    },
}
