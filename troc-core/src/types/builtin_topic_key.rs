use binrw::binrw;
use serde::{Deserialize, Serialize};

use super::InstanceHandle;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[binrw]
#[br(import(_len: usize))]
pub struct BuiltinTopicKey {
    value: InstanceHandle,
}
