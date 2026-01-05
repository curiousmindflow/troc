use std::fmt::Display;

use binrw::binrw;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
pub struct WExtension8(pub(crate) [u8; 8]);

impl Display for WExtension8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.0))?;
        Ok(())
    }
}
