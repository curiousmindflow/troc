use crate::types::Guid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Serialization failed")]
    SerializationError,
    #[error("Deserialization failed")]
    DeserializationError,
    #[error("The Gap submessage is invalid")]
    InvalidGapError,
    #[error("The incomming message has been filtered out, reason: {because}")]
    FilteredOut { because: String },
    #[error("The incomming message comes from un unknown Reader, guid: {0}")]
    RemoteReaderNotFound(Guid),
    #[error("The incomming message comes from un unknown Writer, guid: {0}")]
    RemoteWriterNotFound(Guid),
    #[error("The Writer is in BestEffort mode")]
    IsBestEffort,
}
