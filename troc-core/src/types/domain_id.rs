use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[br(import(_len: usize))]
pub struct DomainId(pub u32);

impl From<u32> for DomainId {
    fn from(value: u32) -> Self {
        DomainId(value)
    }
}

impl Display for DomainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))?;
        Ok(())
    }
}
