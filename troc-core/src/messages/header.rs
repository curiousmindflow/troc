use std::mem::size_of;

use binrw::binrw;

use crate::types::{
    guid::GuidPrefix, protocol_id::ProtocolId, protocol_version::ProtocolVersion,
    vendor_id::VendorId,
};

// 20 bytes
#[derive(Debug, Default, Clone, PartialEq)]
#[binrw]
#[brw(big)]
pub struct Header {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

impl Header {
    pub fn size() -> usize {
        size_of::<Self>()
    }
}
