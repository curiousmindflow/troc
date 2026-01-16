use std::marker::PhantomData;

use bytes::BytesMut;

use crate::types::{
    ContentNature, InlineQos, Timestamp,
    change_count::ChangeCount,
    checksum::Checksum32,
    count::Count,
    fragment_number::FragmentNumber,
    fragment_number_set::FragmentNumberSet,
    guid::{ENTITYID_UNKOWN, EntityId, GuidPrefix},
    locator_list::LocatorList,
    message_length::MessageLength,
    parameter_list::ParameterList,
    protocol_id::ProtocolId,
    protocol_version::ProtocolVersion,
    sequence_number::SequenceNumber,
    sequence_number_set::SequenceNumberSet,
    serialized_data::SerializedData,
    u_extension4::UExtension4,
    vendor_id::VendorId,
    w_extension8::WExtension8,
};

use super::{
    header::Header,
    message::Message,
    submessages::{
        submessage::Submessage,
        submessage_content::{GapGroupInfo, HeartbeatGroupInfo},
    },
};

#[derive(Debug, Clone)]
pub struct MessageFactory {
    guid_prefix: GuidPrefix,
}

impl MessageFactory {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        Self { guid_prefix }
    }

    pub fn message(&mut self) -> MessageBuilder<WriterUnknown, ReaderUnknown> {
        MessageBuilder {
            guid_prefix: self.guid_prefix,
            submessages: Default::default(),
            big_endian: false,
            reader_id: ENTITYID_UNKOWN,
            writer_id: ENTITYID_UNKOWN,
            phantom_w: PhantomData,
            phantom_r: PhantomData,
        }
    }
}

pub struct WriterUnknown;
pub struct WriterKnown;

pub struct ReaderUnknown;
pub struct ReaderKnown;

pub struct MessageBuilder<W, R> {
    guid_prefix: GuidPrefix,
    submessages: Vec<Submessage>,
    big_endian: bool,
    reader_id: EntityId,
    writer_id: EntityId,
    phantom_w: PhantomData<W>,
    phantom_r: PhantomData<R>,
}

impl<W, R> MessageBuilder<W, R> {
    pub fn big_endian(mut self) -> Self {
        self.big_endian = true;
        self
    }

    pub fn little_endian(mut self) -> Self {
        self.big_endian = false;
        self
    }
}

impl<R> MessageBuilder<WriterUnknown, R> {
    pub fn writer(mut self, writer_id: EntityId) -> MessageBuilder<WriterKnown, R> {
        self.writer_id = writer_id;

        let Self {
            guid_prefix,
            submessages,
            big_endian,
            reader_id,
            writer_id,
            phantom_r,
            ..
        } = self;

        MessageBuilder {
            guid_prefix,
            submessages,
            big_endian,
            reader_id,
            writer_id,
            phantom_w: PhantomData,
            phantom_r,
        }
    }
}

impl<W> MessageBuilder<W, ReaderUnknown> {
    pub fn reader(mut self, reader_id: EntityId) -> MessageBuilder<W, ReaderKnown> {
        self.reader_id = reader_id;

        let Self {
            guid_prefix,
            submessages,
            big_endian,
            reader_id,
            writer_id,
            phantom_w,
            ..
        } = self;

        MessageBuilder {
            guid_prefix,
            submessages,
            big_endian,
            reader_id,
            writer_id,
            phantom_w,
            phantom_r: PhantomData,
        }
    }
}

impl MessageBuilder<WriterKnown, ReaderKnown> {
    pub fn data(
        mut self,
        content_nature: ContentNature,
        writer_sn: SequenceNumber,
        inline_qos: Option<InlineQos>,
        serialized_data: Option<SerializedData>,
    ) -> Self {
        let inline_qos = inline_qos.map(|inline_qos| InlineQos::to_parameter_list(&inline_qos));
        let sub = Submessage::data(
            self.big_endian,
            content_nature,
            self.reader_id,
            self.writer_id,
            writer_sn,
            inline_qos,
            serialized_data,
        );
        self.submessages.push(sub);
        self
    }

    pub fn acknack(mut self, writer_sn_state: SequenceNumberSet, count: Count) -> Self {
        let sub = Submessage::acknack(
            self.big_endian,
            self.reader_id,
            self.writer_id,
            writer_sn_state,
            count,
        );
        self.submessages.push(sub);
        self
    }

    pub fn gap(
        mut self,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
        gap_group_info: Option<GapGroupInfo>,
        filtered_count: Option<ChangeCount>,
    ) -> Self {
        let sub = Submessage::gap(
            self.big_endian,
            self.reader_id,
            self.writer_id,
            gap_start,
            gap_list,
            gap_group_info,
            filtered_count,
        );
        self.submessages.push(sub);
        self
    }

    pub fn heartbeat(
        mut self,
        is_final: bool,
        liveliness: bool,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        count: Count,
        heartbeat_group_info: Option<HeartbeatGroupInfo>,
    ) -> Self {
        let sub = Submessage::heartbeat(
            self.big_endian,
            is_final,
            liveliness,
            self.reader_id,
            self.writer_id,
            first_sn,
            last_sn,
            count,
            heartbeat_group_info,
        );
        self.submessages.push(sub);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn datafrag(
        mut self,
        is_key: bool,
        writer_sn: SequenceNumber,
        fragment_starting_num: FragmentNumber,
        fragment_in_submessage: u16,
        sample_size: u32,
        fragment_size: u16,
        inline_qos: Option<InlineQos>,
        serialized_payload: SerializedData,
    ) -> Self {
        let inline_qos = inline_qos.map(|inline_qos| InlineQos::to_parameter_list(&inline_qos));
        let sub = Submessage::datafrag(
            self.big_endian,
            is_key,
            self.reader_id,
            self.writer_id,
            writer_sn,
            fragment_starting_num,
            fragment_in_submessage,
            fragment_size,
            sample_size,
            inline_qos,
            serialized_payload,
        );
        self.submessages.push(sub);
        self
    }

    pub fn heartbeatfrag(
        mut self,
        writer_sn: SequenceNumber,
        last_fragmentation_num: FragmentNumber,
        count: Count,
    ) -> Self {
        let sub = Submessage::heartbeatfrag(
            self.big_endian,
            self.reader_id,
            self.writer_id,
            writer_sn,
            last_fragmentation_num,
            count,
        );
        self.submessages.push(sub);
        self
    }

    pub fn nackfrag(
        mut self,
        writer_sn: SequenceNumber,
        fragmentation_number_state: FragmentNumberSet,
        count: Count,
    ) -> Self {
        let sub = Submessage::nackfrag(
            self.big_endian,
            self.reader_id,
            self.writer_id,
            writer_sn,
            fragmentation_number_state,
            count,
        );
        self.submessages.push(sub);
        self
    }

    pub fn info_timestamp(mut self, timestamp: Option<Timestamp>) -> Self {
        let sub = Submessage::info_timestamp(self.big_endian, timestamp);
        self.submessages.push(sub);
        self
    }

    pub fn info_reply(
        mut self,
        unicast_locator_list: LocatorList,
        multicast_locator_list: Option<LocatorList>,
    ) -> Self {
        let sub = Submessage::info_reply(
            self.big_endian,
            unicast_locator_list,
            multicast_locator_list,
        );
        self.submessages.push(sub);
        self
    }

    pub fn info_destination(mut self, guid_prefix: GuidPrefix) -> Self {
        let sub = Submessage::info_destination(self.big_endian, guid_prefix);
        self.submessages.push(sub);
        self
    }

    pub fn info_source(
        mut self,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
    ) -> Self {
        let sub =
            Submessage::info_source(self.big_endian, protocol_version, vendor_id, guid_prefix);
        self.submessages.push(sub);
        self
    }

    pub fn header_extension(
        mut self,
        message_length: Option<MessageLength>,
        rtps_send_timestamp: Option<Timestamp>,
        u_extension4: Option<UExtension4>,
        w_extension8: Option<WExtension8>,
        message_checksum: Option<Checksum32>,
        parameters: Option<ParameterList>,
    ) -> Self {
        let sub = Submessage::header_extension(
            self.big_endian,
            message_length,
            rtps_send_timestamp,
            u_extension4,
            w_extension8,
            message_checksum,
            parameters,
        );
        self.submessages.push(sub);
        self
    }

    pub fn pad(mut self) -> Self {
        let sub = Submessage::pad();
        self.submessages.push(sub);
        self
    }

    pub fn build(self) -> Message {
        Message {
            header: Header {
                protocol: ProtocolId::default(),
                version: ProtocolVersion::V2_4,
                vendor_id: VendorId::default(),
                guid_prefix: self.guid_prefix,
            },
            submessages: self.submessages,
        }
    }
}

#[cfg(test)]
mod message_tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};
    use rstest::rstest;

    use crate::{
        messages::{message::Message, message_factory::MessageFactory},
        types::{
            ContentNature,
            count::Count,
            guid::{EntityId, GuidPrefix},
            sequence_number::SequenceNumber,
            serialized_data::SerializedData,
            vendor_id::VendorId,
        },
    };

    static DUMMY_VENDOR_ID: VendorId = VendorId([3, 6]);
    static DUMMY_GUID_PREFIX: GuidPrefix = GuidPrefix::from([0u8; 12], DUMMY_VENDOR_ID);
    static DUMMY_ENTITY_ID: EntityId = EntityId {
        entity_key: [252, 253, 254],
        entity_kind: 255,
    };
    static DUMMY_SEQUENCE_NUMBER: SequenceNumber = SequenceNumber(1234);

    #[rstest]
    fn build_message() {
        let mut message_factory = MessageFactory::new(DUMMY_GUID_PREFIX);

        let expected = message_factory
            .message()
            .writer(DUMMY_ENTITY_ID)
            .reader(DUMMY_ENTITY_ID)
            .big_endian()
            .data(
                ContentNature::Data,
                DUMMY_SEQUENCE_NUMBER,
                None,
                Some(SerializedData::from_vec(vec![0u8; 4])),
            )
            .heartbeat(
                false,
                false,
                DUMMY_SEQUENCE_NUMBER,
                DUMMY_SEQUENCE_NUMBER + 1,
                Count::default(),
                None,
            )
            .build();
        let msg_len = expected.size() as usize;

        let mut buf = Cursor::new(Vec::<u8>::new());

        expected.write(&mut buf).unwrap();

        buf.set_position(0);

        let actual = Message::read_args(&mut buf, (msg_len,)).unwrap();

        assert_eq!(expected, actual);
    }
}
