use troc_core::types::Locator;

use crate::io::wire::{TransmissionKind, Wire};

pub struct IpcWire {
    //
}

impl Wire for IpcWire {
    async fn recv_from(
        &self,
        buf: &mut [u8],
    ) -> Result<(usize, Locator), super::wire::ReceptionError> {
        todo!()
    }

    async fn send(&self, buf: &[u8]) -> Result<usize, super::wire::EmissionError> {
        todo!()
    }

    fn transmission_kind(&self) -> TransmissionKind {
        todo!()
    }
}
