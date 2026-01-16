use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

pub static DURATION_ZERO: Duration = Duration {
    seconds: 0,
    fraction: 0,
};

pub static DURATION_INFINITE: Duration = Duration {
    seconds: 0x7fffffff,
    fraction: 0xfffffffe,
};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[binrw]
#[br(import(_len: usize))]
pub struct Duration {
    seconds: i32,
    fraction: u32,
}

impl From<std::time::Duration> for Duration {
    fn from(value: std::time::Duration) -> Self {
        let seconds = value.as_secs() as i32;
        let fraction = ((value.subsec_nanos() as u64) * (1u64 << 32) / 1_000_000_000) as u32;
        Self { seconds, fraction }
    }
}

impl From<Duration> for std::time::Duration {
    fn from(value: Duration) -> Self {
        let seconds = value.seconds as u64;
        let fraction = (((value.fraction as u64) * 1_000_000_000) >> 32) as u32;
        std::time::Duration::new(seconds, fraction)
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{")?;
        f.write_str(&format!("seconds: {}", self.seconds))?;
        f.write_str(&format!("fraction: {}", self.fraction))?;
        f.write_str("}")?;
        Ok(())
    }
}
