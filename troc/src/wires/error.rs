use protocol::types::{Guid, SequenceNumber};
use thiserror::Error;

#[derive(Debug, Error, Default)]
pub enum WireError {
    #[error("Reception error: {0}")]
    ReceptionError(String),
    #[error("Send error: {0}")]
    SendError(String),
    #[error("{0}")]
    CreationError(#[from] std::io::Error),
    #[default]
    #[error("UnkownError")]
    Unkown,
}
