use std::{
    fmt::Display,
    mem::size_of,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use binrw::{BinRead, BinWrite, binrw};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub static TIME_ZERO: TimestampWire = TimestampWire {
    seconds: 0,
    fractions: 0,
};

pub static TIME_INVALID: TimestampWire = TimestampWire {
    seconds: 0xffff,
    fractions: 0xffff,
};

pub static TIME_INFINITE: TimestampWire = TimestampWire {
    seconds: 0xffff,
    fractions: 0xfffe,
};

#[derive(Debug, Clone, Copy)]
#[binrw]
pub struct TimestampWire {
    pub seconds: i32,
    pub fractions: u32,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    BinRead,
    BinWrite,
)]
#[bw(map = Timestamp::write_map)]
#[br(map = Timestamp::read_parse)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn from_datetime(datetime: DateTime<Utc>) -> Timestamp {
        Self(datetime)
    }

    pub fn to_datetime(self) -> DateTime<Utc> {
        self.0
    }

    pub fn to_bytes(self) -> [u8; 8] {
        // let timestamp_wire =
        // let mut bytes = vec![];
        // bytes.append(&mut self.seconds.to_ne_bytes().to_vec());
        // bytes.append(&mut self.fractions.to_ne_bytes().to_vec());
        // let bytes: [u8; 8] = bytes.try_into().unwrap();
        // bytes
        todo!()
    }

    pub fn size(&self) -> usize {
        size_of::<i64>()
    }

    fn write_map(datetime: &Timestamp) -> TimestampWire {
        let seconds = datetime.0.timestamp() as i32;
        let fractions = datetime.0.timestamp_subsec_nanos();
        TimestampWire { seconds, fractions }
    }

    fn read_parse(timestamp_wire: TimestampWire) -> Timestamp {
        let timestamp = DateTime::<Utc>::from_timestamp(
            timestamp_wire.seconds as i64,
            timestamp_wire.fractions,
        )
        .unwrap_or_default();
        Timestamp(timestamp)
    }
}

impl From<TimestampWire> for Timestamp {
    fn from(timestamp_wire: TimestampWire) -> Self {
        Timestamp::read_parse(timestamp_wire)
    }
}

// impl Sub for Timestamp {
//     type Output = Timestamp;

//     fn sub(self, rhs: Self) -> Self::Output {
//         Self {
//             seconds: self.seconds - rhs.seconds,
//             fractions: self.fractions - rhs.fractions,
//         }
//     }
// }

// impl SubAssign for Timestamp {
//     fn sub_assign(&mut self, rhs: Self) {
//         self.seconds = self.seconds - rhs.seconds;
//         self.fractions = self.fractions - rhs.fractions;
//     }
// }

// impl Add for Timestamp {
//     type Output = Timestamp;

//     fn add(self, rhs: Self) -> Self::Output {
//         Self {
//             seconds: self.seconds + rhs.seconds,
//             fractions: self.fractions + rhs.fractions,
//         }
//     }
// }

// impl AddAssign for Timestamp {
//     fn add_assign(&mut self, rhs: Self) {
//         self.seconds = self.seconds + rhs.seconds;
//         self.fractions = self.fractions + rhs.fractions;
//     }
// }

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))?;
        Ok(())
    }
}
