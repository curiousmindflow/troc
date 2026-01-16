use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

use binrw::binrw;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[binrw]
#[br(import(_len: usize))]
pub struct Count(i32);

impl Count {
    pub fn new(c: i32) -> Self {
        Self(c)
    }
}

impl Display for Count {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))?;
        Ok(())
    }
}

impl Default for Count {
    fn default() -> Self {
        Count(-1)
    }
}

impl Add<i32> for Count {
    type Output = Count;

    fn add(self, rhs: i32) -> Self::Output {
        Count(self.0 + rhs)
    }
}

impl AddAssign<i32> for Count {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs;
    }
}
