use std::{
    collections::{BTreeSet, HashMap, hash_map::Entry},
    fmt::{Debug, Display},
};

use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    messages::Message,
    types::{
        Count, FragmentNumber, FragmentNumberSet, Locator,
        guid::{EntityId, Guid},
        locator_list::LocatorList,
        sequence_number::{SEQUENCENUMBER_INVALID, SEQUENCENUMBER_UNKNOWN, SequenceNumber},
    },
};

use crate::{Reader, common::Counter, publication::WriterHistoryCache};

#[derive(Default, Serialize, Deserialize)]
pub struct ReaderProxy {
    pub(crate) remote_reader_guid: Guid,
    pub(crate) remote_group_entity_id: EntityId,
    pub(crate) expects_inline_qos: bool,
    pub(crate) unicast_locator_list: LocatorList,
    pub(crate) multicast_locator_list: LocatorList,
    #[serde(skip)]
    pub(crate) highest_sent_change_sn: SequenceNumber,
    #[serde(skip)]
    pub(crate) requested_changes: BTreeSet<SequenceNumber>,
    #[serde(skip)]
    pub(crate) requested_fragments: HashMap<SequenceNumber, BTreeSet<FragmentNumber>>,
    #[serde(skip)]
    pub(crate) acknowledged_changes: BTreeSet<SequenceNumber>,
    pub(crate) is_active: bool,
    #[serde(skip)]
    pub(crate) last_acknack_timestamp_ms: i64,
    #[serde(skip)]
    pub(crate) acknack_count: Count,
    #[serde(skip)]
    pub(crate) nackfrag_count: Count,
}

impl ReaderProxy {
    pub(crate) fn new(
        remote_reader_guid: Guid,
        remote_group_entity_id: EntityId,
        expects_inline_qos: bool,
        is_active: bool,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            expects_inline_qos,
            unicast_locator_list,
            multicast_locator_list,
            highest_sent_change_sn: SEQUENCENUMBER_UNKNOWN,
            requested_changes: Default::default(),
            requested_fragments: Default::default(),
            acknowledged_changes: Default::default(),
            is_active,
            last_acknack_timestamp_ms: Utc::now().timestamp_millis(),
            acknack_count: Count::default(),
            nackfrag_count: Count::default(),
        }
    }

    pub fn get_locators(&self) -> LocatorList {
        let locators: Vec<Locator> = self
            .unicast_locator_list
            .iter()
            .chain(self.multicast_locator_list.iter())
            .cloned()
            .collect();
        LocatorList::new(locators)
    }

    pub fn get_remote_reader_guid(&self) -> Guid {
        self.remote_reader_guid
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    pub fn can_send(&self) -> bool {
        true
    }

    pub fn get_highest_sent_change_sn(&self) -> SequenceNumber {
        self.highest_sent_change_sn
    }

    pub fn set_highest_sent_change_sn(&mut self, seq: SequenceNumber) {
        self.highest_sent_change_sn = seq;
    }

    pub fn unacked_changes(&self, highest_available_seq_num: SequenceNumber) -> bool {
        let highest_acked_seq_num = self
            .acknowledged_changes
            .first()
            .unwrap_or(&SEQUENCENUMBER_INVALID);
        highest_available_seq_num > *highest_acked_seq_num
    }

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        // let last = *self.acknowledged_changes.iter().max().unwrap();
        // let extension = (last.0..committed_seq_num.0).map(SequenceNumber);
        // self.acknowledged_changes.extend(extension);
        self.acknowledged_changes.clear();
        self.acknowledged_changes.insert(committed_seq_num);
    }

    pub fn get_acknowledged_changes(&self) -> BTreeSet<SequenceNumber> {
        self.acknowledged_changes.clone()
    }

    pub fn next_unsent_change(&self, cache: &WriterHistoryCache, guid: Guid) -> SequenceNumber {
        // let unsent_changes = cache.get_sequence_numbers(guid);
        // let unsent_changes: Vec<SequenceNumber> = unsent_changes
        //     .into_iter()
        //     .filter(|seq| *seq > self.highest_sent_change_sn)
        //     .collect();

        // if unsent_changes.is_empty() {
        //     SEQUENCENUMBER_INVALID
        // } else {
        //     unsent_changes.into_iter().min().unwrap()
        // }
        todo!()
    }

    pub fn unsent_changes(&self, cache: &WriterHistoryCache, guid: Guid) -> bool {
        self.next_unsent_change(cache, guid) != SEQUENCENUMBER_INVALID
    }

    pub fn next_requested_change(&self) -> Option<SequenceNumber> {
        self.requested_changes.first().cloned()
    }

    pub fn remove_last_requested_change(&mut self) {
        self.requested_changes.pop_first();
    }

    pub fn are_requested_changes(&self) -> bool {
        !self.requested_changes.is_empty()
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: Vec<SequenceNumber>) {
        self.requested_changes.extend(req_seq_num_set);
    }

    pub fn are_requested_fragments(&self, sequence: SequenceNumber) -> bool {
        self.requested_fragments.contains_key(&sequence)
    }

    pub fn pop_next_requested_fragment(
        &mut self,
        sequence: SequenceNumber,
    ) -> Option<FragmentNumber> {
        if let Some(frag) = self
            .requested_fragments
            .get_mut(&sequence)
            .unwrap()
            .pop_first()
        {
            Some(frag)
        } else {
            self.requested_fragments.remove(&sequence);
            None
        }
    }

    pub fn requested_fragments_set(
        &mut self,
        writer_sn: SequenceNumber,
        frag_set: Vec<FragmentNumber>,
    ) {
        let set = match self.requested_fragments.entry(writer_sn) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(BTreeSet::new()),
        };
        for frag in frag_set {
            set.insert(frag);
        }
    }

    pub(crate) fn update(&mut self, other: ReaderProxy) {
        self.expects_inline_qos = other.expects_inline_qos;
        self.unicast_locator_list = other.unicast_locator_list;
        self.multicast_locator_list = other.multicast_locator_list;
        self.is_active = other.is_active;
    }

    pub(crate) fn from_base_reader(reader: &Reader) -> Self {
        // Self {
        //     remote_reader_guid: reader.get_guid(),
        //     remote_group_entity_id: reader.remote_group_entity_id(),
        //     expects_inline_qos: reader.get_expects_inline_qos(),
        //     unicast_locator_list: reader.get_unicast_locator_list(),
        //     multicast_locator_list: reader.get_multicast_locator_list(),
        //     is_active: true,
        //     ..Default::default()
        // }
        todo!()
    }
}

impl Clone for ReaderProxy {
    fn clone(&self) -> Self {
        Self {
            remote_reader_guid: self.remote_reader_guid,
            remote_group_entity_id: self.remote_group_entity_id,
            expects_inline_qos: self.expects_inline_qos,
            unicast_locator_list: self.unicast_locator_list.clone(),
            multicast_locator_list: self.multicast_locator_list.clone(),
            highest_sent_change_sn: SEQUENCENUMBER_UNKNOWN,
            requested_changes: Default::default(),
            requested_fragments: Default::default(),
            acknowledged_changes: Default::default(),
            is_active: self.is_active,
            last_acknack_timestamp_ms: Default::default(),
            acknack_count: Default::default(),
            nackfrag_count: Default::default(),
        }
    }
}

impl PartialEq for ReaderProxy {
    fn eq(&self, other: &Self) -> bool {
        self.remote_reader_guid == other.remote_reader_guid
            && self.remote_group_entity_id == other.remote_group_entity_id
    }
}

impl Display for ReaderProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let requested_changes = self.requested_changes.iter().collect::<Vec<_>>();
        let acknowledged_changes = self.acknowledged_changes.iter().collect::<Vec<_>>();
        f.write_str(&format!("ReaderProxy: {{ RemoteReaderGuid: {}, HighestSentChange: {}, RequestedChanges: {:?}, AcknowledgeChanges: {:?}, IsActive: {}, UniLocators: {}, MultiLocators: {}, RemoteGroupEntityId: {}, ExpectsQos: {} }}", self.remote_reader_guid, self.highest_sent_change_sn, requested_changes, acknowledged_changes, self.is_active, self.unicast_locator_list, self.multicast_locator_list, self.remote_group_entity_id, self.expects_inline_qos))?;
        Ok(())
    }
}

impl Debug for ReaderProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReaderProxy")
            .field("remote_reader_guid", &self.remote_reader_guid)
            .field("remote_group_entity_id", &self.remote_group_entity_id)
            .field("expects_inline_qos", &self.expects_inline_qos)
            .field("unicast_locator_list", &self.unicast_locator_list)
            .field("multicast_locator_list", &self.multicast_locator_list)
            .field("highest_sent_change_sn", &self.highest_sent_change_sn)
            .field("requested_changes", &self.requested_changes)
            .field("requested_fragments", &self.requested_fragments)
            .field("acknowledged_changes", &self.acknowledged_changes)
            .field("is_active", &self.is_active)
            .finish()
    }
}
