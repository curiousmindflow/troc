mod error;
mod udpv4_wire;
mod wire;
mod wire_factory;

use async_trait::async_trait;
use bytes::BytesMut;
pub use error::WireError;
use troc_core::Locator;
pub use wire::*;
pub use wire_factory::{
    ReceiverWireActor, ReceiverWireFactoryActorMessage, ReceiverWireFactoryActorMessageDestKind,
    Sendable, SenderWireActor, SenderWireActorMessage, SenderWireFactoryActorMessage,
    WireFactoryActor,
};

// #[cfg(test)]
// use mockall::*;

#[derive(Debug, Clone, Copy)]
pub enum TransmissionKind {
    ToOne,
    ToMany,
}

#[derive(Debug, Clone, Copy)]
pub enum TransmissionDirection {
    Listener,
    Sender,
}

// #[cfg_attr(test, automock)]
#[async_trait]
pub trait Wired: Send + Sync {
    async fn recv(&mut self) -> Result<BytesMut, WireError>;
    async fn send(&mut self, msg: BytesMut) -> Result<(), WireError>;
    fn transmission_kind(&self) -> TransmissionKind;
    fn transmission_direction(&self) -> TransmissionDirection;
    fn locator(&self) -> Locator;
    fn duplicate(&self) -> Box<dyn Wired>;
}
