use binrw::{BinResult, binrw};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[binrw]
#[br(import(len: usize))]
pub struct UserDataQosPolicy {
    #[br(count = len)]
    pub value: Vec<u8>,
}
