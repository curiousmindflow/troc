use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default)]
#[repr(u32)]
pub enum ReliabilityKind {
    #[default]
    BestEffort = 1,
    Reliable = 2,
}

impl Display for ReliabilityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReliabilityKind::BestEffort => f.write_str("BestEffort")?,
            ReliabilityKind::Reliable => f.write_str("Reliable")?,
        };
        Ok(())
    }
}
