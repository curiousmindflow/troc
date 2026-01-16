use std::fmt::Display;

#[derive(Debug, Clone, Default)]
pub enum ChangeForReaderStatusKind {
    #[default]
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}

impl Display for ChangeForReaderStatusKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeForReaderStatusKind::Unsent => {
                f.write_str("ChangeForReaderStatusKind::Unsent")?
            }
            ChangeForReaderStatusKind::Unacknowledged => {
                f.write_str("ChangeForReaderStatusKind::Unacknowledged")?
            }
            ChangeForReaderStatusKind::Requested => {
                f.write_str("ChangeForReaderStatusKind::Requested")?
            }
            ChangeForReaderStatusKind::Acknowledged => {
                f.write_str("ChangeForReaderStatusKind::Acknowledged")?
            }
            ChangeForReaderStatusKind::Underway => {
                f.write_str("ChangeForReaderStatusKind::Underway")?
            }
        };
        Ok(())
    }
}
