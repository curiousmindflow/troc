use std::ops::{Deref, DerefMut};

use crate::types::{
    ChangeKind, FragmentNumber, Guid, InlineQos, InstanceHandle, SequenceNumber, SerializedData,
    Timestamp,
};

use crate::subscription::SampleStateKind;

#[derive(Debug, Default, Clone)]
pub struct CacheChangeInfos {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub instance_handle: InstanceHandle,
    pub sequence_number: SequenceNumber,
    pub sample_size: u32,
    pub fragment_size: u16,
    pub fragments_count: u16,
    pub emission_timestamp: Option<Timestamp>,
    pub reception_timestamp: Timestamp,
    pub inline_qos: Option<InlineQos>,
}

#[derive(Debug, Default, Clone)]
pub struct CacheChange {
    pub infos: CacheChangeInfos,
    pub data: Option<SerializedData>,
}

impl CacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        fragment_size: u16,
        reception_timestamp: Timestamp,
        data: Option<SerializedData>,
    ) -> Self {
        let sample_size = data.as_ref().map(|d| d.size() as u32).unwrap_or_default();
        let fragments_count = data
            .as_ref()
            .map(|d| d.size().div_ceil(fragment_size as usize))
            .unwrap_or_default() as u16;
        Self {
            infos: CacheChangeInfos {
                kind,
                writer_guid,
                instance_handle,
                sequence_number,
                sample_size,
                fragment_size,
                fragments_count,
                emission_timestamp: None,
                reception_timestamp,
                inline_qos: None,
            },
            data,
        }
    }

    pub fn set_qos(&mut self, qos: InlineQos) {
        self.infos.inline_qos.replace(qos);
    }

    pub fn set_emission_timestamp(&mut self, emission_timestamp: Timestamp) {
        self.infos.emission_timestamp.replace(emission_timestamp);
    }

    pub fn get_sequence_number(&self) -> SequenceNumber {
        self.infos.sequence_number
    }

    pub fn get_guid(&self) -> Guid {
        self.infos.writer_guid
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.infos.instance_handle
    }

    pub fn get_inline_qos(&self) -> Option<&InlineQos> {
        self.infos.inline_qos.as_ref()
    }

    pub fn get_emission_timestamp(&self) -> Option<Timestamp> {
        self.infos.emission_timestamp
    }

    pub fn get_data(&self) -> Option<&SerializedData> {
        self.data.as_ref()
    }

    pub fn get_reception_timestamp(&self) -> Timestamp {
        self.infos.reception_timestamp
    }
}

impl From<FragmentedCacheChange> for CacheChange {
    fn from(value: FragmentedCacheChange) -> Self {
        Self {
            infos: value.infos,
            data: Some(SerializedData::from_vec(value.data)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheChangeContainer {
    change: CacheChange,
    readed: SampleStateKind,
}

impl CacheChangeContainer {
    pub fn new(change: CacheChange) -> Self {
        Self {
            change,
            readed: SampleStateKind::NotRead,
        }
    }

    pub fn get_sample_state_kind(&self) -> SampleStateKind {
        self.readed
    }

    pub fn mark_read(&mut self) {
        self.readed = SampleStateKind::Read;
    }

    pub fn into_inner(self) -> CacheChange {
        self.change
    }
}

impl Deref for CacheChangeContainer {
    type Target = CacheChange;

    fn deref(&self) -> &Self::Target {
        &self.change
    }
}

impl DerefMut for CacheChangeContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.change
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FragPresence {
    Present,
    Missing,
}

#[derive(Debug, Default, Clone)]
pub struct FragmentedCacheChange {
    pub infos: CacheChangeInfos,
    pub frags_index: Vec<FragPresence>,
    pub data: Vec<u8>,
}

impl FragmentedCacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        fragment_size: u16,
        reception_timestamp: Timestamp,
        sample_size: u32,
    ) -> Self {
        let fragments_count = sample_size.div_ceil(fragment_size as u32) as u16;
        Self {
            infos: CacheChangeInfos {
                kind,
                writer_guid,
                instance_handle,
                sequence_number,
                sample_size,
                fragment_size,
                fragments_count,
                emission_timestamp: None,
                reception_timestamp,
                inline_qos: None,
            },
            frags_index: vec![FragPresence::Missing; fragments_count as usize],
            data: Vec::with_capacity(sample_size as usize),
        }
    }

    pub fn set_qos(&mut self, qos: InlineQos) {
        self.infos.inline_qos.replace(qos);
    }

    pub fn set_emission_timestamp(&mut self, emission_timestamp: Timestamp) {
        self.infos.emission_timestamp.replace(emission_timestamp);
    }

    pub fn get_sequence_number(&self) -> SequenceNumber {
        self.infos.sequence_number
    }

    pub fn get_guid(&self) -> Guid {
        self.infos.writer_guid
    }

    pub fn check_validity(&self, fragment_size: u16, sample_size: u32) -> Result<(), ()> {
        // TODO: assert fragments infos (frag size, sample size, etc ...) are coherents with those already recorded, if not, return an error
        Ok(())
    }

    /// Check if some fragments are
    ///
    /// # Panic
    /// Panic if the targeted [`FragmentedCacheChange`] is not in cache
    pub fn are_fragments_missing(
        &self,
        fragment_starting_num: FragmentNumber,
        frags_count: u16,
    ) -> bool {
        self.frags_index
            .iter()
            .skip(fragment_starting_num.0 as usize - 1)
            .take(frags_count as usize)
            .all(|presence| matches!(presence, FragPresence::Present))
    }

    pub fn get_missing_fragments_number(&self) -> Vec<FragmentNumber> {
        self.frags_index
            .iter()
            .enumerate()
            .filter(|(_, presence)| matches!(presence, FragPresence::Missing))
            .map(|(i, _)| FragmentNumber(i as u32 + 1))
            .collect()
    }

    /// Insert one or many consecutive fragments in the [`SerializedData`] fot he corresponding [`FragmentedCacheChange`]
    ///
    /// # Panic
    /// Panic if the targeted [`FragmentedCacheChange`] is not in cache
    pub fn insert_fragments(
        &mut self,
        fragment_starting_num: FragmentNumber,
        frags_count: u16,
        fragment_size: u16,
        frags: &SerializedData,
    ) {
        let total_fragments_size = (frags_count * fragment_size) as usize;
        self.data
            .get_mut(fragment_starting_num.0 as usize..total_fragments_size)
            .unwrap()
            .copy_from_slice(frags.get_data());
        self.frags_index
            .iter_mut()
            .skip(fragment_starting_num.0 as usize - 1)
            .take(frags_count as usize)
            .for_each(|presence| *presence = FragPresence::Present);
    }

    pub fn is_complete(&self) -> bool {
        self.frags_index
            .iter()
            .all(|presence| matches!(presence, FragPresence::Present))
    }
}
