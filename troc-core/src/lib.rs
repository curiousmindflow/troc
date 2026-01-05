mod common;
mod discovery;
mod key;
pub mod messages;
mod publication;
mod subscription;
pub mod types;

pub use cdr::{CdrBe, Infinite};
pub use common::{
    CacheChange, CacheChangeInfos, Effect, Effects, IncommingMessage, OutcommingMessage,
    ReaderProxy, WriterProxy,
};
pub use discovery::{
    Announce, DiscoveredReaderData, DiscoveredWriterData, Discovery, DiscoveryBuilder,
    DiscoveryConfiguration,
};
pub use key::{KeyCalculationError, Keyed};
use pretty_hex::HexConfig;
pub use publication::{Writer, WriterBuilder, WriterConfiguration};
use serde::{Deserialize, Serialize};
pub use subscription::{Reader, ReaderBuilder, ReaderConfiguration};
use thiserror::Error;
use types::SerializedData;

#[derive(Debug, Error)]
pub enum DdsError {
    #[error("Generic, unspecified error. Additional infos: {0}")]
    Error(String),
    #[error("Illegal parameter value.")]
    BadParameter,
    #[error(
        "Unsupported operation. Can only be returned by operations
    that are optional."
    )]
    Unsupported,
    #[error(
        "The object target of this operation has already been
    deleted."
    )]
    AlreadyDeleted,
    #[error(
        " Service ran out of the resources needed to complete the
    operation."
    )]
    OutOfResources,
    #[error("Operation invoked on an Entity that is not yet enabled.")]
    NotEnabled,
    #[error("Application attempted to modify an immutable QosPolicy.")]
    ImmutablePolicy,
    #[error(
        "Application specified a set of policies that are not
    consistent with each other."
    )]
    InconsistentPolicy,
    #[error("A pre-condition for the operation was not met.")]
    PreconditionNotMet,
    #[error("The operation timed out. Cause: {cause}")]
    Timeout { cause: String },
    #[error(
        "An operation was invoked on an inappropriate object or at
    an inappropriate time (as determined by policies set by the
    specification or the Service implementation). There is no
    precondition that could be changed to make the operation
    succeed. "
    )]
    IllegalOperation,
    #[error(
        " Indicates a transient situation where the operation did not
    return any data but there is no inherent error. "
    )]
    NoData,
}

pub const K: u32 = 1024;
pub const M: u32 = K * 1024;

const PRETTY_HEX_CONFIG: HexConfig = HexConfig {
    title: true,
    ascii: true,
    width: 0,
    group: 0,
    chunk: 0,
    max_bytes: usize::MAX,
    display_offset: 0,
};

pub fn serialize_data<T>(data: &T) -> Result<SerializedData, cdr::Error>
where
    T: Serialize,
{
    let data = cdr::serialize::<_, _, CdrBe>(data, Infinite)?;
    let data = SerializedData::from_vec(data);
    Ok(data)
}

pub fn deserialize_data<'a, T>(data: &SerializedData) -> Result<T, cdr::Error>
where
    T: Deserialize<'a>,
{
    cdr::deserialize::<T>(&data.get_data())
}
