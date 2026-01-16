use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

pub static LOCATOR_KIND_INVALID: LocatorKind = LocatorKind::Invalid;
pub static LOCATOR_KIND_RESERVED: LocatorKind = LocatorKind::Reserved;
pub static LOCATOR_KIND_UDPV4: LocatorKind = LocatorKind::UdpV4;
pub static LOCATOR_KIND_UDPV6: LocatorKind = LocatorKind::UdpV6;

#[binrw]
#[derive(
    Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[brw(repr(i32))]
pub enum LocatorKind {
    #[default]
    UdpV4 = 1,
    UdpV6 = 2,
    Shm = 16,
    Reserved = 0,
    Invalid = -1,
}

impl Display for LocatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = match self {
            LocatorKind::UdpV4 => "UDPV4",
            LocatorKind::UdpV6 => "UDPV6",
            _ => "OTHER",
        };
        f.write_str(kind_str)?;
        Ok(())
    }
}
