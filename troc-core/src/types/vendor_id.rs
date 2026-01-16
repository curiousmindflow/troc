use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

pub static VENDORID_UNKNOWN: VendorId = VendorId([0, 0]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[binrw]
#[br(import(_len: usize))]
pub struct VendorId(pub(crate) [u8; 2]);

impl Display for VendorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.0))?;
        Ok(())
    }
}

impl Default for VendorId {
    fn default() -> Self {
        VENDORID_UNKNOWN
    }
}
