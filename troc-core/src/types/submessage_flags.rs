use binrw::{BinRead, BinWrite};
use modular_bitfield::{bitfield, specifiers::B1};

#[bitfield]
#[derive(Clone, Copy, Default, Debug, BinRead, BinWrite, PartialEq, Eq, PartialOrd, Ord)]
#[br(map = Self::from_bytes)]
#[bw(map = |&x| Self::into_bytes(x))]
pub struct SubmessageFlags {
    pub e: B1,
    pub second: B1,
    pub third: B1,
    pub fourth: B1,
    pub fifth: B1,
    pub sixth: B1,
    pub seventh: B1,
    pub eighth: B1,
}
