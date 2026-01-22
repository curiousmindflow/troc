use std::{
    collections::{HashMap, btree_map::Entry},
    fmt::{Debug, Display},
};

use chrono::Utc;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::types::{
    EntityId, FragmentNumber, Locator, SEQUENCENUMBER_INVALID,
    change_count::ChangeCount,
    change_from_writer_status_kind::ChangeFromWriterStatusKind,
    guid::Guid,
    locator_list::LocatorList,
    sequence_number::{SEQUENCENUMBER_UNKNOWN, SequenceNumber},
};

use crate::{
    Writer,
    common::Counter,
    subscription::{ChangeFromWriter, ChangeFromWriterMap},
};

#[derive(Default, Serialize, Deserialize)]
pub struct WriterProxy {
    pub(crate) remote_writer_guid: Guid,
    pub(crate) remote_group_entity_id: EntityId,
    pub(crate) unicast_locator_list: LocatorList,
    pub(crate) multicast_locator_list: LocatorList,
    pub(crate) data_max_size_serialized: u32,
    #[serde(skip)]
    pub(crate) changes_from_writer_map: ChangeFromWriterMap,
    #[serde(skip)]
    pub(crate) missing_frags: HashMap<SequenceNumber, FragmentNumber>,
    #[serde(skip)]
    pub(crate) last_heartbeat_timestamp_ms: i64,
    #[serde(skip)]
    pub(crate) acknack_counter: Counter,
    #[serde(skip)]
    pub(crate) nackfrag_counter: Counter,
}

impl WriterProxy {
    pub(crate) fn new(
        remote_writer_guid: Guid,
        remote_group_entity_id: EntityId,
        data_max_size_serialized: u32,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
    ) -> Self {
        Self {
            remote_writer_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
            changes_from_writer_map: Default::default(),
            missing_frags: Default::default(),
            last_heartbeat_timestamp_ms: Utc::now().timestamp_millis(),
            acknack_counter: Counter::new(),
            nackfrag_counter: Counter::new(),
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

    pub fn get_remote_writer_guid(&self) -> Guid {
        self.remote_writer_guid
    }

    pub fn last_readed_change(&self) -> SequenceNumber {
        unimplemented!()
    }

    pub fn available_changes_pack(&self) -> Vec<(SequenceNumber, bool)> {
        let available_changes_max = self.available_changes_max();
        self.changes_from_writer_map
            .changes_from_writer
            .iter()
            .filter(move |&(s, c)| {
                *s <= available_changes_max && c.status == ChangeFromWriterStatusKind::Received
            })
            .map(|(s, cfw)| (*s, cfw.readed))
            .collect()
    }

    pub fn available_changes(&self) -> Vec<SequenceNumber> {
        let available_changes_max = self.available_changes_max();
        self.changes_from_writer_map
            .changes_from_writer
            .iter()
            .filter(|(_, c)| {
                c.change_sequence_number <= available_changes_max
                    && c.status == ChangeFromWriterStatusKind::Received
            })
            .map(|(s, _)| *s)
            .collect()
    }

    pub fn available_changes_max(&self) -> SequenceNumber {
        self.changes_from_writer_map
            .changes_from_writer
            .iter()
            .take_while(|(_, e)| {
                e.status == ChangeFromWriterStatusKind::Received
                    || e.status == ChangeFromWriterStatusKind::NotAvailableFiltered
                    || e.status == ChangeFromWriterStatusKind::NotAvailableRemoved
                    || e.status == ChangeFromWriterStatusKind::NotAvailableUnspecified
            })
            .map(|(_, e)| e.change_sequence_number)
            .last()
            .unwrap_or(SEQUENCENUMBER_UNKNOWN)
    }

    pub fn not_available_change_set(
        &mut self,
        set: Vec<SequenceNumber>,
        filtered_count: ChangeCount,
    ) {
        let max_seq = set.iter().max_by_key(|s| s.0).cloned().unwrap_or_default();
        self.fill(max_seq);
        self.changes_from_writer_map
            .changes_from_writer
            .iter_mut()
            .filter(|(seq, _e)| set.contains(seq))
            .for_each(|(_, e)| {
                if filtered_count == set.len() {
                    e.status = ChangeFromWriterStatusKind::NotAvailableFiltered;
                } else if filtered_count == 0 {
                    e.status = ChangeFromWriterStatusKind::NotAvailableRemoved;
                } else {
                    e.status = ChangeFromWriterStatusKind::NotAvailableUnspecified
                }
            })
    }

    pub fn lost_changes(&self) -> Vec<SequenceNumber> {
        self.changes_from_writer_map
            .changes_from_writer
            .iter()
            .filter_map(|(seq, e)| {
                if e.status == ChangeFromWriterStatusKind::NotAvailableFiltered
                    || e.status == ChangeFromWriterStatusKind::NotAvailableRemoved
                    || e.status == ChangeFromWriterStatusKind::NotAvailableUnspecified
                {
                    Some(*seq)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn lost_changes_update(
        &mut self,
        first_available_seq_num: SequenceNumber,
        changes_removed: bool,
    ) {
        self.fill(first_available_seq_num);
        self.changes_from_writer_map
            .changes_from_writer
            .iter_mut()
            .filter(|(_, e)| {
                e.status == ChangeFromWriterStatusKind::Unknown
                    || e.status == ChangeFromWriterStatusKind::Missing
            })
            .filter(|(seq, _)| **seq < first_available_seq_num)
            .for_each(|(_, e)| {
                if changes_removed {
                    e.status = ChangeFromWriterStatusKind::NotAvailableRemoved
                } else {
                    e.status = ChangeFromWriterStatusKind::NotAvailableUnspecified
                }
            })
    }

    pub fn missing_changes(&self) -> Vec<SequenceNumber> {
        self.changes_from_writer_map
            .changes_from_writer
            .iter()
            .filter_map(|(_, e)| {
                if matches!(e.status, ChangeFromWriterStatusKind::Missing) {
                    Some(e.change_sequence_number)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber) {
        self.fill(last_available_seq_num);
        self.changes_from_writer_map
            .changes_from_writer
            .iter_mut()
            .filter(|(seq, e)| {
                e.status == ChangeFromWriterStatusKind::Unknown && **seq <= last_available_seq_num
            })
            .for_each(|(_, e)| e.status = ChangeFromWriterStatusKind::Missing);
    }

    pub fn last_announced_frag(&self, sequence: SequenceNumber) -> Option<FragmentNumber> {
        self.missing_frags.get(&sequence).copied()
    }

    pub fn set_last_announced_frag(&mut self, sequence: SequenceNumber, frag: FragmentNumber) {
        self.missing_frags.insert(sequence, frag);
    }

    pub fn last_missing_frag_remove(&mut self, sequence: SequenceNumber) {
        self.missing_frags.remove(&sequence);
    }

    pub fn last_missing_frag_remove_until(&mut self, sequence: SequenceNumber) {
        self.missing_frags.retain(|seq, _| *seq > sequence);
    }

    pub fn received_changes(&mut self) -> Vec<SequenceNumber> {
        self.changes_from_writer_map
            .changes_from_writer
            .iter_mut()
            .filter_map(|(seq, change)| {
                if matches!(change.status, ChangeFromWriterStatusKind::Received) {
                    Some(*seq)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn received_change_set(&mut self, seq_num: SequenceNumber) {
        // FIXME: it should set the status of Sequence that are UNKOWN/MISSINGS to Received (not touch those lost/unavailable)
        self.fill(seq_num);
        self.changes_from_writer_map
            .changes_from_writer
            .iter_mut()
            .filter(|(seq, _)| **seq == seq_num)
            .for_each(|(_, change)| change.status = ChangeFromWriterStatusKind::Received);
    }

    pub fn is_change_received(&self, sequence: SequenceNumber) -> bool {
        self.changes_from_writer_map
            .changes_from_writer
            .iter()
            .filter_map(|(seq, c)| {
                if matches!(c.status, ChangeFromWriterStatusKind::Received) {
                    Some(seq)
                } else {
                    None
                }
            })
            .contains(&sequence)
    }

    pub fn expected_sequence(&self) -> SequenceNumber {
        self.available_changes_max() + 1
    }

    fn fill(&mut self, up_to: SequenceNumber) {
        let first_sequence = self
            .changes_from_writer_map
            .changes_from_writer
            .first_key_value()
            .map(|(seq, _)| *seq)
            .unwrap_or(SequenceNumber(1));

        for sequence in first_sequence.0..=up_to.0 + 1 {
            if let Entry::Vacant(vacant) = self
                .changes_from_writer_map
                .changes_from_writer
                .entry(SequenceNumber(sequence))
            {
                vacant.insert(ChangeFromWriter {
                    is_relevant: true,
                    status: ChangeFromWriterStatusKind::Unknown,
                    change_sequence_number: SequenceNumber(sequence),
                    readed: false,
                });
            }
        }
    }

    pub fn clean(&mut self, min_seq_in_cache: &SequenceNumber) {
        fn clean_condition(cfw: &ChangeFromWriter, min_seq_in_cache: &SequenceNumber) -> bool {
            &cfw.change_sequence_number < min_seq_in_cache
                && (matches!(cfw.status, ChangeFromWriterStatusKind::NotAvailableFiltered)
                    || matches!(cfw.status, ChangeFromWriterStatusKind::NotAvailableRemoved)
                    || matches!(
                        cfw.status,
                        ChangeFromWriterStatusKind::NotAvailableUnspecified
                    )
                    || matches!(cfw.status, ChangeFromWriterStatusKind::Received))
        }

        self.changes_from_writer_map
            .changes_from_writer
            .retain(|_, cfw| !clean_condition(cfw, min_seq_in_cache));
    }

    fn display_change_from_writer(&self) -> String {
        self.changes_from_writer_map
            .changes_from_writer
            .values()
            .map(|cfw| cfw.to_string())
            .join(",")
    }

    pub(crate) fn update(&mut self, other: WriterProxy) {
        self.unicast_locator_list = other.unicast_locator_list;
        self.multicast_locator_list = other.multicast_locator_list;
    }

    pub(crate) fn from_base_writer(writer: &Writer) -> Self {
        // Self {
        //     remote_writer_guid: writer.get_guid(),
        //     remote_group_entity_id: writer.remote_group_entity_id(),
        //     unicast_locator_list: writer.get_unicast_locator_list(),
        //     multicast_locator_list: writer.get_multicast_locator_list(),
        //     data_max_size_serialized: writer.get_data_max_size_serialized(),
        //     changes_from_writer_map: Default::default(),
        //     ..Default::default()
        // }
        todo!()
    }
}

impl Clone for WriterProxy {
    fn clone(&self) -> Self {
        Self {
            remote_writer_guid: self.remote_writer_guid,
            data_max_size_serialized: self.data_max_size_serialized,
            changes_from_writer_map: Default::default(),
            remote_group_entity_id: self.remote_group_entity_id,
            unicast_locator_list: self.unicast_locator_list.clone(),
            multicast_locator_list: self.multicast_locator_list.clone(),
            missing_frags: Default::default(),
            last_heartbeat_timestamp_ms: Default::default(),
            acknack_counter: Counter::new(),
            nackfrag_counter: Counter::new(),
        }
    }
}

impl PartialEq for WriterProxy {
    fn eq(&self, other: &Self) -> bool {
        self.remote_writer_guid == other.remote_writer_guid
    }
}

impl Display for WriterProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("WriterProxy: {{ RemoteWriterGuid: {}, DataMaxSize: {}, UniLocators: {}, MultiLocators: {}, RemoteGroupEntityId: {} }}", self.remote_writer_guid, self.data_max_size_serialized, self.unicast_locator_list, self.multicast_locator_list, self.remote_group_entity_id))?;
        Ok(())
    }
}

impl Debug for WriterProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriterProxy")
            .field("remote_writer_guid", &self.remote_writer_guid)
            .field("remote_group_entity_id", &self.remote_group_entity_id)
            .field("unicast_locator_list", &self.unicast_locator_list)
            .field("multicast_locator_list", &self.multicast_locator_list)
            .field("data_max_size_serialized", &self.data_max_size_serialized)
            .field("changes_from_writer_map", &self.changes_from_writer_map)
            .field("missing_frags", &self.missing_frags)
            .finish()
    }
}
