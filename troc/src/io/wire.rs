use std::fmt::Display;

use async_trait::async_trait;
use thiserror::Error;
use troc_core::types::Locator;

#[derive(Debug, Error)]
pub struct ReceptionError {
    cause: String,
}

impl Display for ReceptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Error)]
pub struct EmissionError {
    cause: String,
}

impl Display for EmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug)]
pub enum TransmissionKind {
    OneToOne,
    OneToMany,
}

// #[async_trait]
pub trait Wire: Send {
    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, Locator), ReceptionError>;
    async fn send(&self, buf: &[u8]) -> Result<usize, EmissionError>;
    fn transmission_kind(&self) -> TransmissionKind;
}
