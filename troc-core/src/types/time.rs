use std::fmt::Display;

use chrono::TimeZone;

pub static TIME_ZERO: Time = Time {
    seconds: 0,
    fraction: 0,
};

pub static TIME_INVALID: Time = Time {
    seconds: 0xffffffff,
    fraction: 0xffffffff,
};

pub static TIME_INFINITE: Time = Time {
    seconds: 0xffffffff,
    fraction: 0xfffffffe,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Time {
    seconds: u32,
    fraction: u32,
}

impl From<chrono::DateTime<chrono::Utc>> for Time {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        let seconds = value.timestamp() as u32;
        let fraction = value.timestamp_subsec_nanos();
        Self { seconds, fraction }
    }
}

impl From<Time> for chrono::DateTime<chrono::Utc> {
    fn from(value: Time) -> Self {
        chrono::DateTime::from_timestamp(value.seconds as i64, value.fraction).unwrap()
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("seconds: {}", self.seconds))?;
        f.write_str(&format!("fraction: {}", self.fraction))?;
        Ok(())
    }
}
