use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeKind {
    #[default]
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

impl Display for ChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeKind::Alive => f.write_str("ChangeKind::Alive")?,
            ChangeKind::AliveFiltered => f.write_str("ChangeKind::AliveFiltered")?,
            ChangeKind::NotAliveDisposed => f.write_str("ChangeKind::NotAliveDisposed")?,
            ChangeKind::NotAliveUnregistered => f.write_str("ChangeKind::NotAliveUnregistered")?,
        };
        Ok(())
    }
}
