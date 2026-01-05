#![allow(dead_code)]

use super::{guid::Guid, parameter_list::ParameterList, sequence_number::SequenceNumber};

#[derive(Debug, Default, Clone)]
pub struct OriginalWriterInfo {
    original_writer_guid: Guid,
    original_writer_sn: SequenceNumber,
    original_writer_qos: ParameterList,
}
