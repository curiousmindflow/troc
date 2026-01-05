use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[binrw]
#[br(import(_len: usize))]
pub struct InstanceHandle(pub [u8; 16]);

impl InstanceHandle {
    pub fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
}

impl Display for InstanceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex = pretty_hex::simple_hex(&self.0);
        f.write_str(&hex)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use proptest::prelude::*;

    use super::InstanceHandle;

    proptest! {
        #[test]
        fn compare_two_instances(instance_0 in prop::array::uniform16(0u8..)) {
            println!("instance: {:?}", instance_0);
            let key_0 = InstanceHandle::new(instance_0);
            let key_1 = InstanceHandle::new(instance_0);
            assert_eq!(key_0, key_1)
        }
    }
}
