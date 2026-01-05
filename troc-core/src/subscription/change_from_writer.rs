use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};

use crate::types::{
    change_from_writer_status_kind::ChangeFromWriterStatusKind, sequence_number::SequenceNumber,
};

#[derive(Debug, Default, Clone)]
pub struct ChangeFromWriterMap {
    pub(crate) changes_from_writer: BTreeMap<SequenceNumber, ChangeFromWriter>,
}

impl Display for ChangeFromWriterMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_seq, change) in self.changes_from_writer.iter() {
            f.write_str(&format!("{change}\n"))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeFromWriter {
    pub(crate) is_relevant: bool,
    pub(crate) status: ChangeFromWriterStatusKind,
    pub(crate) change_sequence_number: SequenceNumber,
    pub(crate) readed: bool,
}

impl Display for ChangeFromWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let relevant = if self.is_relevant { "r" } else { "i" };
        f.write_str(&format!(
            "{} [{}] - {}",
            self.change_sequence_number, relevant, self.status
        ))?;
        Ok(())
    }
}
