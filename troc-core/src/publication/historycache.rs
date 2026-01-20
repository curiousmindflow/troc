use std::collections::VecDeque;

use crate::types::{FragmentNumber, SequenceNumber};
use contracts::requires;
use thiserror::Error;
use tracing::{Level, event};

use crate::CacheChange;

#[derive(Debug)]
pub struct WriterHistoryCacheConfiguration {
    depth: Option<u32>,
    max_instances: u32,
    max_samples_per_instance: u32,
    max_samples: u32,
}

impl WriterHistoryCacheConfiguration {
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
pub enum WriterHistoryCacheError {
    #[error("The Change's SequenceNumber({0}) is already in use")]
    SequenceAlreadyPresent(SequenceNumber),
}

#[derive(Debug)]
pub struct WriterHistoryCache {
    changes: VecDeque<CacheChange>,
    depth: Option<u32>,
    max_instances: u32,
    max_samples_per_instance: u32,
    max_samples: u32,
    trash: VecDeque<CacheChange>,
}

impl WriterHistoryCache {
    pub fn new(config: WriterHistoryCacheConfiguration) -> Self {
        let WriterHistoryCacheConfiguration {
            depth,
            max_instances,
            max_samples_per_instance,
            max_samples,
        } = config;
        let changes_size = config.depth.unwrap_or(max_samples) as usize;
        let changes = VecDeque::with_capacity(changes_size);
        Self {
            changes,
            depth,
            max_instances,
            max_samples_per_instance,
            max_samples,
            trash: Default::default(),
        }
    }

    /// Push a new CacheChange into the ReaderHistoryCache
    ///
    /// # Preconditions
    /// - change.sequence must not be already in use
    pub fn push_change(&mut self, change: CacheChange) -> Result<(), WriterHistoryCacheError> {
        if let Some(depth) = self.depth {
            if self.changes.len() == depth as usize {
                let taken_change = self.changes.pop_back().expect("presence asserted");
                self.trash.push_front(taken_change);
            }
            self.changes.push_front(change);
            Ok(())
        } else {
            // FIXME: handle resources limit cases
            self.changes.push_front(change);
            Ok(())
        }
    }

    pub fn get_change(&self, sequence: SequenceNumber) -> Option<&CacheChange> {
        self.changes
            .iter()
            .find(|c| c.get_sequence_number() == sequence)
    }

    pub fn get_min_sequence(&self) -> Option<SequenceNumber> {
        self.changes.iter().map(|c| c.get_sequence_number()).min()
    }

    pub fn get_max_sequence(&self) -> Option<SequenceNumber> {
        self.changes.iter().map(|c| c.get_sequence_number()).max()
    }

    pub fn get_last_frag_per_sequence(&self) -> Vec<(SequenceNumber, FragmentNumber)> {
        self.changes
            .iter()
            .map(|c| {
                let seq = c.get_sequence_number();
                let last_frag_num = FragmentNumber(c.infos.fragments_count as u32 - 1);
                (seq, last_frag_num)
            })
            .collect()
    }
}
