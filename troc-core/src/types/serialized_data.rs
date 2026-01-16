#![allow(clippy::useless_conversion)]

use std::{io::Cursor, mem::size_of, ops::Deref, sync::Arc};

use binrw::{BinRead, BinResult, Endian, Error, binrw};
use serde::{Deserialize, Serialize};

use crate::messages::SubmessageHeader;

use super::ParameterList;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[br(import(len: u16))]
pub struct SerializedData {
    #[br(parse_with = serialized_data_parser, args((len.into(),)))]
    #[bw(write_with = serialized_data_writer)]
    data: Arc<Vec<u8>>,
}

impl SerializedData {
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: Arc::new(data.to_vec()),
        }
    }

    pub fn size(&self) -> usize {
        size_of::<u8>() * self.data.len()
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl From<SerializedData> for Arc<Vec<u8>> {
    fn from(value: SerializedData) -> Self {
        value.data
    }
}

impl From<Vec<u8>> for SerializedData {
    fn from(value: Vec<u8>) -> Self {
        SerializedData::from_vec(value)
    }
}

#[binrw::parser(reader)]
fn serialized_data_parser(args: (usize,)) -> BinResult<Arc<Vec<u8>>> {
    let end_position = args.0;
    let current_position: usize = reader.stream_position().unwrap().try_into().unwrap();

    let remaining_bytes = end_position - current_position;
    let mut buf = vec![0u8; remaining_bytes];

    reader.read_exact(&mut buf)?;

    Ok(Arc::new(buf))
}

#[binrw::writer(writer)]
fn serialized_data_writer(data: &Arc<Vec<u8>>) -> BinResult<()> {
    writer.write_all(data)?;
    Ok(())
}
