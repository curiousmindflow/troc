#![allow(dead_code)]
#![allow(clippy::identity_op)]
use std::fmt::Display;

use binrw::{BinRead, BinWrite};
use modular_bitfield::{
    bitfield,
    specifiers::{B1, B31},
};
use serde::{Deserialize, Serialize};

#[bitfield]
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    BinRead,
    BinWrite,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[br(map = Self::from_bytes)]
#[bw(map = |&x| Self::into_bytes(x))]
#[br(import(_len: usize))]
pub struct BuiltinEndpointQos {
    best_effort_participant_messge_data_reader: B1,
    unused: B31,
}

impl Display for BuiltinEndpointQos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("N/A")?;
        Ok(())
    }
}
