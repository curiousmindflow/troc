mod ipc_wire;
mod timer;
mod udpv4_wire;
mod wire;
mod wire_factory;

use std::fmt::Display;

pub use ipc_wire::IpcWire;
use thiserror::Error;
pub use timer::{Timer, TimerHandle};
pub use udpv4_wire::UdpV4Wire;
pub use wire::{EmissionError, ReceptionError, TransmissionKind, Wire};
pub use wire_factory::*;

#[derive(Debug, Error)]
pub struct WireCreationError {
    cause: String,
}

impl WireCreationError {
    pub fn from(e: impl std::error::Error) -> Self {
        Self {
            cause: e.to_string(),
        }
    }
}

impl Display for WireCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cause)
    }
}
