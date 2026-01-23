use super::{header::Header, submessages::submessage::Submessage};
use binrw::{BinRead, BinResult, BinWrite, binrw};
use bytes::{Buf, BufMut, BytesMut};
use std::{
    fmt::Display,
    io::{BufWriter, Cursor, Error},
    mem::size_of,
};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Error)]
pub struct SerializationError {
    source: binrw::Error,
}

impl Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Error)]
pub struct DeserializationError {
    source: binrw::Error,
}

impl Display for DeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
#[binrw]
#[br(import(message_length: usize))]
#[brw(big)]
pub struct Message {
    #[br(parse_with = header_parser)]
    pub header: Header,
    #[br(parse_with = submessages_parser, args((message_length,)))]
    pub submessages: Vec<Submessage>,
}

#[binrw::parser(reader)]
fn header_parser() -> BinResult<Header> {
    let header = Header::read(reader)?;
    Ok(header)
}

#[binrw::parser(reader)]
fn submessages_parser(len: (usize,)) -> BinResult<Vec<Submessage>> {
    let len = len.0;
    let mut remaining = len - Header::size();
    let mut subs = Vec::new();

    while remaining > 0 {
        let sub = Submessage::read(reader)?;
        let sub_size = sub.size();
        remaining -= sub_size;
        subs.push(sub);
    }

    Ok(subs)
}

impl Message {
    pub fn size(&self) -> u64 {
        let mut size = 0;
        size += size_of::<Header>();
        size += self
            .submessages
            .iter()
            .fold(0usize, |acc, sub| acc + sub.size());
        let size: u64 = size.try_into().unwrap();
        size
    }

    pub fn deserialize_from(buffer: &BytesMut) -> Result<Self, DeserializationError> {
        let mut reader = Cursor::new(buffer.as_ref());
        Message::read_args(&mut reader, (buffer.len(),))
            .map_err(|e| DeserializationError { source: e })
    }

    pub fn serialize_to(&self, buffer: &mut BytesMut) -> Result<usize, SerializationError> {
        let mut writer = Cursor::new(buffer.as_mut());
        self.write(&mut writer)
            .map_err(|e| SerializationError { source: e })?;
        let size = writer.position() as usize;
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use binrw::Endian;
    use bytes::{Buf, BufMut, BytesMut};
    use chrono::Utc;
    use rstest::rstest;
    use tokio_util::codec::{Decoder, Encoder};

    use crate::{
        messages::{Message, MessageFactory, SubmessageContent},
        types::{
            ContentNature, Count, ENTITYID_UNKOWN, EntityId, FragmentNumber, GuidPrefix, InlineQos,
            InstanceHandle, ParameterId, ParameterList, SequenceNumber, SequenceNumberSet,
            SerializedData, Timestamp,
        },
    };

    #[rstest]
    fn test_complex_message() {
        let mut buf = BytesMut::zeroed(64 * 1024);
        let mut message_factory = MessageFactory::new(GuidPrefix::default());

        let expected_message = message_factory
            .message()
            .reader(EntityId::default())
            .writer(EntityId::default())
            .info_timestamp(Some(Timestamp::from_datetime(Utc::now())))
            .data(
                ContentNature::Data,
                SequenceNumber::default(),
                None,
                Some(SerializedData::from_slice(&[0u8; 10])),
            )
            .heartbeat(
                true,
                false,
                SequenceNumber(0),
                SequenceNumber(1),
                Count::default(),
                None,
            )
            .build();

        let size = expected_message.serialize_to(&mut buf).unwrap();
        let splitted_buf = buf.split_to(size);

        let actual_message = Message::deserialize_from(&splitted_buf).unwrap();

        assert_eq!(expected_message, actual_message);
    }

    // FIXME: fail because of `inline_qos`
    #[rstest]
    fn test_data_message() {
        let mut buf = BytesMut::zeroed(64 * 1024);
        let mut message_factory = MessageFactory::new(GuidPrefix::default());

        let inline_qos = InlineQos {
            key_hash: InstanceHandle([77; 16]),
            ..Default::default()
        };

        let expected_message = message_factory
            .message()
            .reader(EntityId::default())
            .writer(EntityId::default())
            .data(
                ContentNature::Data,
                SequenceNumber::default(),
                Some(inline_qos),
                Some(SerializedData::from_slice(&[
                    129, 128, 127, 125, 124, 123, 122, 121, 120, 119,
                ])),
            )
            .build();
        let size = expected_message.serialize_to(&mut buf).unwrap();
        let splitted_buf = buf.split_to(size);

        let actual_message = Message::deserialize_from(&splitted_buf).unwrap();

        let SubmessageContent::Data {
            inline_qos: ref expected_qos,
            ..
        } = expected_message.submessages.first().unwrap().content
        else {
            panic!()
        };
        let SubmessageContent::Data {
            inline_qos: ref actual_qos,
            ..
        } = actual_message.submessages.first().unwrap().content
        else {
            panic!()
        };

        dbg!(expected_qos);
        dbg!(actual_qos);

        assert_eq!(expected_qos, actual_qos);
        // assert_eq!(expected_message, actual_message);
    }

    #[rstest]
    fn test_frag_data_message() {
        let mut buf = BytesMut::zeroed(64 * 1024);
        let mut message_factory = MessageFactory::new(GuidPrefix::default());

        let inline_qos = InlineQos {
            key_hash: InstanceHandle([77; 16]),
            ..Default::default()
        };

        let expected_message_0 = message_factory
            .message()
            .reader(EntityId::default())
            .writer(EntityId::default())
            .datafrag(
                false,
                SequenceNumber::default(),
                FragmentNumber(0),
                1,
                61 * 1024,
                60 * 1024,
                Some(inline_qos),
                SerializedData::from_slice(&[0u8; 60 * 1024]),
            )
            .build();

        let size = expected_message_0.serialize_to(&mut buf).unwrap();
        let splitted_buf = buf.split_to(size);

        let actual_message = Message::deserialize_from(&splitted_buf).unwrap();

        assert_eq!(expected_message_0, actual_message);

        // buf.clear();

        let expected_message_1 = message_factory
            .message()
            .reader(EntityId::default())
            .writer(EntityId::default())
            .datafrag(
                false,
                SequenceNumber::default(),
                FragmentNumber(1),
                1,
                61 * 1024,
                60 * 1024,
                None,
                SerializedData::from_slice(&[0u8; 1024]),
            )
            .build();

        let size = expected_message_1.serialize_to(&mut buf).unwrap();
        let splitted_buf = buf.split_to(size);

        let actual_message = Message::deserialize_from(&splitted_buf).unwrap();

        assert_eq!(expected_message_1, actual_message);
    }

    #[rstest]
    fn test_gap_message() {
        let mut buf = BytesMut::zeroed(64 * 1024);
        let mut message_factory = MessageFactory::new(GuidPrefix::default());

        let expected_message = message_factory
            .message()
            .reader(ENTITYID_UNKOWN)
            .writer(ENTITYID_UNKOWN)
            .gap(
                SequenceNumber(0),
                SequenceNumberSet::new(SequenceNumber(0), &[]),
                None,
                None,
            )
            .build();

        let size = expected_message.serialize_to(&mut buf).unwrap();
        let splitted_buf = buf.split_to(size);

        let actual_message = Message::deserialize_from(&splitted_buf).unwrap();

        assert_eq!(expected_message, actual_message);
    }
}
