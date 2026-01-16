use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub},
};

use binrw::binrw;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize,
)]
#[binrw]
pub struct FragmentNumber(pub u32);

impl Display for FragmentNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))?;
        Ok(())
    }
}

impl Add<FragmentNumber> for FragmentNumber {
    type Output = FragmentNumber;

    fn add(self, rhs: FragmentNumber) -> Self::Output {
        FragmentNumber(self.0 + rhs.0)
    }
}

impl AddAssign<FragmentNumber> for FragmentNumber {
    fn add_assign(&mut self, rhs: FragmentNumber) {
        self.0 += rhs.0
    }
}

impl Add<usize> for FragmentNumber {
    type Output = FragmentNumber;

    fn add(self, rhs: usize) -> Self::Output {
        let rhs: u32 = rhs.try_into().unwrap();
        FragmentNumber(self.0 + rhs)
    }
}

impl AddAssign<usize> for FragmentNumber {
    fn add_assign(&mut self, rhs: usize) {
        let rhs: u32 = rhs.try_into().unwrap();
        self.0 += rhs;
    }
}

impl AddAssign<u64> for FragmentNumber {
    fn add_assign(&mut self, rhs: u64) {
        let rhs: u32 = rhs.try_into().unwrap();
        self.0 += rhs;
    }
}

impl Add<u64> for FragmentNumber {
    type Output = FragmentNumber;

    fn add(self, rhs: u64) -> Self::Output {
        let rhs: u32 = rhs.try_into().unwrap();
        FragmentNumber(self.0 + rhs)
    }
}

impl Sub<FragmentNumber> for FragmentNumber {
    type Output = FragmentNumber;

    fn sub(self, rhs: FragmentNumber) -> Self::Output {
        FragmentNumber(self.0 - rhs.0)
    }
}

impl From<u32> for FragmentNumber {
    fn from(value: u32) -> Self {
        FragmentNumber(value)
    }
}

impl From<FragmentNumber> for u32 {
    fn from(value: FragmentNumber) -> Self {
        value.0
    }
}
