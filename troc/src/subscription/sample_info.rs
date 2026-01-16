use troc_core::{CacheChange, CacheChangeInfos};
use troc_core::{InstanceHandle, Timestamp};

use crate::subscription::sample_state_kind::SampleStateKind;

use super::{instance_state_kind::InstanceStateKind, view_state_kind::ViewStateKind};

#[derive(Debug, Default, Clone, Copy)]
pub struct SampleInfo {
    pub valid_data: bool,
    pub source_timestamp: Timestamp,
    pub sample_state: SampleStateKind,
    pub view_state: ViewStateKind,
    pub instance_handle: InstanceHandle,
    pub instance_state: InstanceStateKind,
    pub disposed_generation_count: i32,
    pub no_writers_generation_cont: i32,
    pub absolute_generation_rank: i32,
    pub sample_rank: i32,
    pub generation_rank: i32,
    pub publication_handle: InstanceHandle,
}

impl From<&CacheChangeInfos> for SampleInfo {
    fn from(change: &CacheChangeInfos) -> Self {
        let valid_data = !change.sample_size.eq(&0);
        let mut source_timestamp = Default::default();
        let sample_state = Default::default();
        let view_state = Default::default();
        let instance_handle = change.instance_handle;
        let instance_state = Default::default();
        let disposed_generation_count = Default::default();
        let no_writers_generation_cont = Default::default();
        let absolute_generation_rank = Default::default();
        let sample_rank = Default::default();
        let generation_rank = Default::default();
        let publication_handle = instance_handle;

        if let Some(qos) = &change.inline_qos {
            // source_timestamp = qos.get_timestamp();
            // TODO: get other values from QoS: StatusInfo (must be implemented in RTPS)
        }

        SampleInfo {
            valid_data,
            source_timestamp,
            sample_state,
            view_state,
            instance_handle,
            instance_state,
            disposed_generation_count,
            no_writers_generation_cont,
            absolute_generation_rank,
            sample_rank,
            generation_rank,
            publication_handle,
        }
    }
}
