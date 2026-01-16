use std::mem::size_of;

use binrw::binrw;

use crate::types::{submessage_flags::SubmessageFlags, submessage_kind::SubmessageKind};

// 8.3.3.3
// 4 bytes
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[brw(big)]
pub struct SubmessageHeader {
    pub submessage_id: SubmessageKind,
    pub flags: SubmessageFlags,
    #[brw(is_big = flags.e() == 0)]
    pub submessage_length: u16,
}

impl SubmessageHeader {
    pub fn size() -> usize {
        let mut size = 0;
        size += size_of::<SubmessageKind>();
        size += size_of::<SubmessageFlags>();
        size += size_of::<u16>();
        size
    }
}
