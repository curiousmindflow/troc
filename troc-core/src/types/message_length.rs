use std::fmt::Display;

use binrw::binrw;

pub static MESSAGE_LENGTH_INVALID: MessageLength = MessageLength(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
pub struct MessageLength(pub(crate) u32);

impl Display for MessageLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))?;
        Ok(())
    }
}

impl Default for MessageLength {
    fn default() -> Self {
        MESSAGE_LENGTH_INVALID
    }
}
