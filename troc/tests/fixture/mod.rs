pub(crate) mod fixtures;

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use troc::{DDSType, KeyCalculationError, Keyed, cdr};

#[allow(unused_imports)]
pub use fixtures::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, DDSType)]
pub struct DummyStruct {
    #[key]
    pub id: u8,
    pub content: Vec<u8>,
}

impl DummyStruct {
    pub fn new(id: u8, content: &[u8]) -> Self {
        Self {
            id,
            content: content.to_vec(),
        }
    }
}

impl Display for DummyStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "id: {}, content size: {}",
            self.id,
            self.content.len()
        ))?;
        Ok(())
    }
}
