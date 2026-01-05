use binrw::{BinRead, BinWrite};
use modular_bitfield::{bitfield, specifiers::B32};

#[bitfield]
#[derive(Default, Clone, Copy, BinRead, BinWrite)]
#[br(map = Self::from_bytes)]
#[bw(map = |&x| Self::into_bytes(x))]
pub struct SubmessageInfo {
    pub none: B32,
}
