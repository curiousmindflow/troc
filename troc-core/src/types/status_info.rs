#![allow(clippy::identity_op)]

use modular_bitfield::{
    bitfield,
    specifiers::{B1, B29},
};
use serde::{Deserialize, Serialize};

#[bitfield]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct StatusInfo {
    #[skip]
    __: B29,
    pub filter: B1,
    pub unregister: B1,
    pub disposed: B1,
}

impl StatusInfo {
    pub fn new_alive() -> Self {
        Self::default()
    }

    pub fn new_disposed() -> Self {
        let mut status = Self::default();
        status.set_disposed(1);
        status
    }

    pub fn new_unregister() -> Self {
        let mut status = Self::default();
        status.set_unregister(1);
        status
    }
}
