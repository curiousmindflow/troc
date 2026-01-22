use std::collections::{HashMap, VecDeque};

use crate::types::{Guid, InstanceHandle, SequenceNumber};
use contracts::requires;
use thiserror::Error;

use crate::{
    CacheChange,
    common::{CacheChangeContainer, FragmentedCacheChange},
};

#[derive(Debug)]
pub struct ReaderHistoryCacheConfiguration {
    depth: Option<u32>,
    max_instances: u32,
    max_samples_per_instance: u32,
    max_samples: u32,
}

impl ReaderHistoryCacheConfiguration {
    #[requires(depth.is_some() -> depth.unwrap() <= max_samples_per_instance)]
    #[requires(max_samples >= max_samples_per_instance)]
    #[requires(max_instances > 0)]
    pub fn new(
        depth: Option<u32>,
        max_instances: u32,
        max_samples_per_instance: u32,
        max_samples: u32,
    ) -> Self {
        Self {
            depth,
            max_instances,
            max_samples_per_instance,
            max_samples,
        }
    }
}

#[derive(Debug, Error)]
pub enum ReaderHistoryCacheError {
    #[error("The Change's SequenceNumber({0}) is already in use")]
    SequenceAlreadyPresent(SequenceNumber),
    #[error("The FragmentedCacheChange<guid:{writer_guid}, sequence:{sequence}> is not in cache")]
    FragmentedCacheChangeAbsent {
        writer_guid: Guid,
        sequence: SequenceNumber,
    },
}

#[derive(Debug)]
pub struct ReaderHistoryCache {
    changes: VecDeque<CacheChangeContainer>,
    frag_changes: HashMap<(SequenceNumber, Guid), FragmentedCacheChange>,
    depth: Option<u32>,
    max_instances: u32,
    max_samples_per_instance: u32,
    max_samples: u32,
    trash: VecDeque<CacheChange>,
}

impl ReaderHistoryCache {
    pub fn new(config: ReaderHistoryCacheConfiguration) -> Self {
        let ReaderHistoryCacheConfiguration {
            depth,
            max_instances,
            max_samples_per_instance,
            max_samples,
        } = config;
        let changes_size = config.depth.unwrap_or(max_samples) as usize;
        let changes = VecDeque::with_capacity(changes_size);
        let frag_changes = HashMap::with_capacity(changes_size);
        let trash = VecDeque::with_capacity(1);
        Self {
            changes,
            frag_changes,
            depth,
            max_instances,
            max_samples_per_instance,
            max_samples,
            trash,
        }
    }

    /// Push a new CacheChange into the ReaderHistoryCache
    ///
    /// # Preconditions
    /// - change.sequence must not be already in use
    pub fn push_change(&mut self, change: CacheChange) -> Result<(), ReaderHistoryCacheError> {
        let change = CacheChangeContainer::new(change);
        if let Some(depth) = self.depth {
            if self.changes.len() == depth as usize {
                let taken_change = self.changes.pop_back().expect("presence asserted");
                self.trash.push_front(taken_change.into_inner());
            }
            self.changes.push_front(change);
            Ok(())
        } else {
            // FIXME: handle resources limit cases
            self.changes.push_front(change);
            Ok(())
        }
    }

    pub fn get_changes(&self) -> impl Iterator<Item = &CacheChangeContainer> {
        self.changes.iter()
    }

    pub fn get_change(
        &self,
        writer_guid: Guid,
        sequence: SequenceNumber,
    ) -> Option<&CacheChangeContainer> {
        self.changes
            .iter()
            .find(|c| c.get_guid() == writer_guid && c.get_sequence_number() == sequence)
    }

    pub fn get_changes_by_instance(&self, instance: InstanceHandle) -> Vec<&CacheChangeContainer> {
        self.changes
            .iter()
            .filter(|c| c.get_instance_handle() == instance)
            .collect()
    }

    pub fn take_change(
        &mut self,
        writer_guid: Guid,
        sequence: SequenceNumber,
    ) -> Option<CacheChangeContainer> {
        let pos = self
            .changes
            .iter()
            .position(|c| c.get_guid() == writer_guid && c.get_sequence_number() == sequence)?;
        self.changes.remove(pos)
    }

    /// Push a new empty [`FragmentedCacheChangeContainer`] inside an inner working storage
    ///
    /// # Preconditions
    /// - `change` must be absent from the cache
    pub fn push_fragmented_change(&mut self, change: FragmentedCacheChange) {
        let old = self
            .frag_changes
            .insert((change.get_sequence_number(), change.get_guid()), change);
        assert!(old.is_none());
    }

    pub fn get_fragmented_change(
        &self,
        writer_guid: Guid,
        sequence: SequenceNumber,
    ) -> Option<&FragmentedCacheChange> {
        self.frag_changes.get(&(sequence, writer_guid))
    }

    pub fn get_fragmented_change_mut(
        &mut self,
        writer_guid: Guid,
        sequence: SequenceNumber,
    ) -> Option<&mut FragmentedCacheChange> {
        self.frag_changes.get_mut(&(sequence, writer_guid))
    }

    /// Remove a FragmentedCacheChange from the cache
    /// If the change doesn't exists, this method do nothing
    pub fn remove_fragmented_change(
        &mut self,
        writer_guid: Guid,
        sequence: SequenceNumber,
    ) -> Option<FragmentedCacheChange> {
        self.frag_changes.remove(&(sequence, writer_guid))
    }

    pub fn transfer(
        &mut self,
        writer_guid: Guid,
        sequence: SequenceNumber,
    ) -> Result<(), ReaderHistoryCacheError> {
        let Some(frag_change) = self.remove_fragmented_change(writer_guid, sequence) else {
            return Err(ReaderHistoryCacheError::FragmentedCacheChangeAbsent {
                writer_guid,
                sequence,
            });
        };
        let change = frag_change.into();
        self.push_change(change)
    }

    /// Remove an entry from the trash and return it if any
    /// This method will return Some(change) until the trash is empty
    ///
    /// # Example
    ///
    // /// ```no_run
    /// let config = ReaderHistoryCacheConfiguration::new(Some(1), 10, 10, 10);
    /// let mut cache = ReaderHistoryCache::new(config);
    ///
    /// cache.push(CacheChange::default()).unwrap();
    /// cache.push(CacheChange::default()).unwrap();
    ///
    /// let garbage = cache.collect_garbage();
    /// assert!(garbage.is_some());
    ///
    /// ```
    pub fn collect_garbage(&mut self) -> Option<CacheChange> {
        self.trash.pop_back()
    }
}
