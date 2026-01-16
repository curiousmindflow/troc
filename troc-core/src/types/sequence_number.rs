use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use binrw::{BinRead, BinWrite, binrw};
use serde::{Deserialize, Serialize};

#[binrw]
struct SequenceNumberWire {
    high: i32,
    low: u32,
}

pub static SEQUENCENUMBER_UNKNOWN: SequenceNumber = SequenceNumber(0);
pub static SEQUENCENUMBER_INVALID: SequenceNumber = SequenceNumber(-4294967296i64); // 0xFFFFFFFF00000000

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Hash,
    BinRead,
    BinWrite,
)]
#[bw(map = SequenceNumber::write_map)]
#[br(map = SequenceNumber::read_parse)]
pub struct SequenceNumber(pub i64);

impl SequenceNumber {
    fn write_map(seq: &SequenceNumber) -> SequenceNumberWire {
        let seq_be_bytes = seq.0.to_be_bytes();
        let high: [u8; 4] = (&seq_be_bytes[0..4]).try_into().unwrap();
        let high = i32::from_be_bytes(high);
        let low: [u8; 4] = (&seq_be_bytes[4..]).try_into().unwrap();
        let low = u32::from_be_bytes(low);
        SequenceNumberWire { high, low }
    }

    fn read_parse(value: SequenceNumberWire) -> SequenceNumber {
        let mut seq_be_bytes = [0u8; 8];
        let high = value.high.to_be_bytes();
        let low = value.low.to_be_bytes();
        seq_be_bytes[0..4].clone_from_slice(&high);
        seq_be_bytes[4..].clone_from_slice(&low);
        let seq = i64::from_be_bytes(seq_be_bytes);
        SequenceNumber(seq)
    }
}

impl Display for SequenceNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))?;
        Ok(())
    }
}

impl AddAssign<i64> for SequenceNumber {
    fn add_assign(&mut self, rhs: i64) {
        self.0 += rhs;
    }
}

impl Add<i64> for SequenceNumber {
    type Output = SequenceNumber;

    fn add(self, rhs: i64) -> Self::Output {
        SequenceNumber(self.0 + rhs)
    }
}

impl SubAssign<i64> for SequenceNumber {
    fn sub_assign(&mut self, rhs: i64) {
        self.0 -= rhs;
    }
}

impl Sub<i64> for SequenceNumber {
    type Output = SequenceNumber;

    fn sub(self, rhs: i64) -> Self::Output {
        SequenceNumber(self.0 - rhs)
    }
}

impl Sub<SequenceNumber> for SequenceNumber {
    type Output = SequenceNumber;

    fn sub(self, rhs: SequenceNumber) -> Self::Output {
        SequenceNumber(self.0 - rhs.0)
    }
}
