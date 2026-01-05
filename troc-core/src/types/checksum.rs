use binrw::binrw;

pub static CHECKSUM32_INVALID: Checksum32 = Checksum32([0; 4]);
pub static CHECKSUM64_INVALID: Checksum64 = Checksum64([0; 8]);
pub static CHECKSUM128_INVALID: Checksum128 = Checksum128([0; 16]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
pub struct Checksum32(pub(crate) [u8; 4]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
pub struct Checksum64(pub(crate) [u8; 8]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
pub struct Checksum128(pub(crate) [u8; 16]);
