use std::{fmt::Display, ops::Add};

use binrw::binrw;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
pub struct ChangeCount(pub(crate) u64);

impl Display for ChangeCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))?;
        Ok(())
    }
}

impl Add<usize> for ChangeCount {
    type Output = ChangeCount;

    fn add(self, rhs: usize) -> Self::Output {
        let rhs: u64 = rhs.try_into().unwrap();
        ChangeCount(self.0 + rhs)
    }
}

impl PartialEq<usize> for ChangeCount {
    fn eq(&self, other: &usize) -> bool {
        let other: u64 = (*other).try_into().unwrap();
        self.0 == other
    }
}
