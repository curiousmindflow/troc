use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum ChangeFromWriterStatusKind {
    #[default]
    Unknown,
    Missing,
    Received,
    Underway,
    NotAvailableFiltered,
    NotAvailableRemoved,
    NotAvailableUnspecified,
}

impl Display for ChangeFromWriterStatusKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeFromWriterStatusKind::Unknown => f.write_str("Unknown")?,
            ChangeFromWriterStatusKind::Missing => f.write_str("Missing")?,
            ChangeFromWriterStatusKind::Received => f.write_str("Received")?,
            ChangeFromWriterStatusKind::Underway => f.write_str("Underway")?,
            ChangeFromWriterStatusKind::NotAvailableFiltered => {
                f.write_str("NotAvailableFiltered")?
            }
            ChangeFromWriterStatusKind::NotAvailableRemoved => {
                f.write_str("NotAvailableRemoved")?
            }
            ChangeFromWriterStatusKind::NotAvailableUnspecified => {
                f.write_str("NotAvailableUnspecified")?
            }
        };
        Ok(())
    }
}
