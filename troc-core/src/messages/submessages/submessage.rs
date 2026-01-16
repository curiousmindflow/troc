use std::mem::{size_of, size_of_val};

use binrw::{BinRead, BinResult, BinWrite, binrw};

use crate::types::{
    ContentNature, Timestamp,
    change_count::ChangeCount,
    checksum::Checksum32,
    count::Count,
    fragment_number::FragmentNumber,
    fragment_number_set::FragmentNumberSet,
    guid::{EntityId, GuidPrefix},
    locator_list::LocatorList,
    message_length::MessageLength,
    parameter_list::ParameterList,
    protocol_version::ProtocolVersion,
    sequence_number::SequenceNumber,
    sequence_number_set::SequenceNumberSet,
    serialized_data::SerializedData,
    submessage_extra_flags::SubmessageExtraFlags,
    submessage_flags::SubmessageFlags,
    u_extension4::UExtension4,
    vendor_id::VendorId,
    w_extension8::WExtension8,
};

use super::{
    submessage_content::{GapGroupInfo, HeartbeatGroupInfo, SubmessageContent},
    submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[binrw]
#[brw(big, stream = s)]
pub struct Submessage {
    pub header: SubmessageHeader,
    #[brw(is_big = header.flags.e() == 0, args(header.flags, header.submessage_length, header.submessage_id, s.stream_position().unwrap().try_into().unwrap()))]
    pub content: SubmessageContent,
}

impl Submessage {
    pub fn data(
        big_endian: bool,
        content_nature: ContentNature,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        inline_qos: Option<ParameterList>,
        serialized_data: Option<SerializedData>,
    ) -> Self {
        let mut flags = SubmessageFlags::new()
            .with_e(big_endian as u8)
            .with_second(inline_qos.is_some() as u8);

        match content_nature {
            ContentNature::Data => flags.set_third(true as u8),
            ContentNature::Key => flags.set_fourth(true as u8),
            ContentNature::None => (),
        }

        let extra_flags = SubmessageExtraFlags::new();
        // FIXME: 9.4.5.4.3 octetsToInlineQos
        // it seems that 'octetsToInlineQos' is always a fixed size no matter the content of it
        let octets_to_inline_qos = if flags.second() == 1 {
            let value = size_of_val(&reader_id) + size_of_val(&writer_id) + size_of_val(&writer_sn);
            value as u16
        } else {
            16
        };

        let content = SubmessageContent::Data {
            extra_flags,
            octets_to_inline_qos,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos: inline_qos.clone(),
            serialized_data,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn acknack(
        big_endian: bool,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn_state: SequenceNumberSet,
        count: Count,
    ) -> Self {
        let flags = SubmessageFlags::new().with_e(big_endian as u8);

        let content = SubmessageContent::AckNack {
            reader_id,
            writer_id,
            writer_sn_state,
            count,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn gap(
        big_endian: bool,
        reader_id: EntityId,
        writer_id: EntityId,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
        gap_group_info: Option<GapGroupInfo>,
        filtered_count: Option<ChangeCount>,
    ) -> Self {
        let flags = SubmessageFlags::new()
            .with_e(big_endian as u8)
            .with_second(gap_group_info.is_some() as u8)
            .with_third(filtered_count.is_some() as u8);

        let content = SubmessageContent::Gap {
            reader_id,
            writer_id,
            gap_start,
            gap_list,
            gap_group_info,
            filtered_count,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn heartbeat(
        big_endian: bool,
        is_final: bool,
        liveliness: bool,
        reader_id: EntityId,
        writer_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        count: Count,
        heartbeat_group_info: Option<HeartbeatGroupInfo>,
    ) -> Self {
        let flags = SubmessageFlags::new()
            .with_e(big_endian as u8)
            .with_second(is_final as u8)
            .with_third(liveliness as u8)
            .with_fourth(heartbeat_group_info.is_some() as u8);

        let content = SubmessageContent::Heartbeat {
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            heartbeat_group_info,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn datafrag(
        big_endian: bool,
        is_key: bool,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        fragment_starting_num: FragmentNumber,
        fragment_in_submessage: u16,
        fragment_size: u16,
        sample_size: u32,
        inline_qos: Option<ParameterList>,
        serialized_payload: SerializedData,
    ) -> Self {
        let flags = SubmessageFlags::new()
            .with_e(big_endian as u8)
            .with_second(inline_qos.is_some() as u8)
            .with_fourth(is_key as u8);

        let extra_flags = SubmessageFlags::new();
        let octets_to_inline_qos = if flags.second() == 1 {
            let value = size_of_val(&reader_id)
                + size_of_val(&writer_id)
                + size_of_val(&writer_sn)
                + size_of_val(&fragment_starting_num)
                + size_of_val(&fragment_in_submessage)
                + size_of_val(&fragment_size)
                + size_of_val(&sample_size)
                + 1;
            value as u16
        } else {
            24
        };

        let content = SubmessageContent::DataFrag {
            flags,
            extra_flags,
            octets_to_inline_qos,
            reader_id,
            writer_id,
            writer_sn,
            fragment_starting_num,
            fragment_in_submessage,
            fragment_size,
            sample_size,
            inline_qos,
            serialized_payload,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn heartbeatfrag(
        big_endian: bool,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        last_fragmentation_num: FragmentNumber,
        count: Count,
    ) -> Self {
        let flags = SubmessageFlags::new().with_e(big_endian as u8);

        let content = SubmessageContent::HeartbeatFrag {
            reader_id,
            writer_id,
            writer_sn,
            last_fragmentation_num,
            count,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn nackfrag(
        big_endian: bool,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        fragmentation_number_state: FragmentNumberSet,
        count: Count,
    ) -> Self {
        let flags = SubmessageFlags::new().with_e(big_endian as u8);

        let content = SubmessageContent::NackFrag {
            reader_id,
            writer_id,
            writer_sn,
            fragmentation_number_state,
            count,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn info_timestamp(big_endian: bool, timestamp: Option<Timestamp>) -> Self {
        let flags = SubmessageFlags::new()
            .with_e(big_endian as u8)
            .with_second(timestamp.is_none() as u8);

        let content = SubmessageContent::InfoTimestamp { timestamp };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn info_reply(
        big_endian: bool,
        unicast_locator_list: LocatorList,
        multicast_locator_list: Option<LocatorList>,
    ) -> Self {
        let flags = SubmessageFlags::new()
            .with_e(big_endian as u8)
            .with_second(multicast_locator_list.is_some() as u8);

        let content = SubmessageContent::InfoReply {
            unicast_locator_list,
            multicast_locator_list,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn info_destination(big_endian: bool, guid_prefix: GuidPrefix) -> Self {
        let flags = SubmessageFlags::new().with_e(big_endian as u8);

        let content = SubmessageContent::InfoDestination { guid_prefix };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn info_source(
        big_endian: bool,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
    ) -> Self {
        let flags = SubmessageFlags::new().with_e(big_endian as u8);

        let content = SubmessageContent::InfoSource {
            protocol_version,
            vendor_id,
            guid_prefix,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn header_extension(
        big_endian: bool,
        message_length: Option<MessageLength>,
        rtps_send_timestamp: Option<Timestamp>,
        u_extension4: Option<UExtension4>,
        w_extension8: Option<WExtension8>,
        message_checksum: Option<Checksum32>,
        parameters: Option<ParameterList>,
    ) -> Self {
        let flags = SubmessageFlags::new()
            .with_e(big_endian as u8)
            .with_second(message_length.is_some() as u8)
            .with_third(rtps_send_timestamp.is_some() as u8)
            .with_fourth(u_extension4.is_some() as u8)
            .with_fifth(w_extension8.is_some() as u8)
            .with_sixth(message_checksum.is_some() as u8)
            .with_seventh(message_checksum.is_some() as u8)
            .with_eighth(parameters.is_some() as u8);

        let content = SubmessageContent::HeaderExtension {
            message_length,
            rtps_send_timestamp,
            u_extension4,
            w_extension8,
            message_checksum,
            parameters,
        };

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn pad() -> Self {
        let flags = SubmessageFlags::new();

        let content = SubmessageContent::Pad;

        let submessage_id = content.id();
        let submessage_length = content.size().try_into().unwrap();

        let header = SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        };

        Self { header, content }
    }

    pub fn size(&self) -> usize {
        size_of::<SubmessageHeader>() + self.content.size()
    }
}

#[cfg(test)]
mod submessages_tests {
    #![allow(clippy::too_many_arguments)]
    #![allow(dead_code)]

    use binrw::{BinRead, BinWrite};
    use rstest::{fixture, rstest};
    use std::io::Cursor;

    use crate::{
        messages::submessages::{
            submessage_content::{GapGroupInfo, HeartbeatGroupInfo},
            submessage_header::SubmessageHeader,
        },
        types::{
            ContentNature, Timestamp,
            change_count::ChangeCount,
            checksum::Checksum32,
            count::Count,
            fragment_number::FragmentNumber,
            fragment_number_set::FragmentNumberSet,
            guid::{EntityId, GuidPrefix},
            locator::Locator,
            locator_kind::LocatorKind,
            locator_list::LocatorList,
            message_length::MessageLength,
            parameter_list::ParameterList,
            protocol_version::ProtocolVersion,
            sequence_number::SequenceNumber,
            sequence_number_set::SequenceNumberSet,
            serialized_data::SerializedData,
            submessage_flags::SubmessageFlags,
            submessage_kind::SubmessageKind,
            u_extension4::UExtension4,
            vendor_id::VendorId,
            w_extension8::WExtension8,
        },
    };

    use super::Submessage;

    static READER_ENTITY_ID: EntityId = EntityId {
        entity_key: [1, 1, 1],
        entity_kind: 1,
    };
    static WRITER_ENTITY_ID: EntityId = EntityId {
        entity_key: [2, 2, 2],
        entity_kind: 2,
    };
    static SEQUENCE_NUMBER: SequenceNumber = SequenceNumber(1234);
    static SUB_HEADER_LEN: usize = 4;
    static DUMMY_VENDOR_ID: VendorId = VendorId([3, 6]);
    static DUMMY_GUID_PREFIX: GuidPrefix = GuidPrefix::from([0u8; 12], DUMMY_VENDOR_ID);

    struct Checker;
    impl Checker {
        fn check(&self, expected: Submessage) {
            let mut buf = Cursor::new(Vec::<u8>::new());

            expected.write(&mut buf).unwrap();

            buf.set_position(0);

            let actual = Submessage::read(&mut buf).unwrap();

            assert_eq!(expected, actual);
        }
    }

    #[fixture]
    fn injected_checker() -> Checker {
        Checker {}
    }

    #[rstest]
    #[case(16, vec![21, 0b00000000, 0, 16])]
    fn sub_header_serialization_deserialization(
        #[case] len: u16,
        #[case] serialized_repr_expected: Vec<u8>,
    ) {
        let mut buf = Cursor::new(Vec::<u8>::new());

        let expected = SubmessageHeader {
            submessage_id: SubmessageKind::Data,
            flags: SubmessageFlags::new(),
            submessage_length: len,
        };

        expected.write(&mut buf).unwrap();

        let mut serialized_repr_actual = Cursor::new(Vec::<u8>::new());
        buf.clone_into(&mut serialized_repr_actual);
        let serialized_repr_actual = serialized_repr_actual.into_inner();

        assert_eq!(serialized_repr_expected, serialized_repr_actual);

        buf.set_position(0);

        let actual = SubmessageHeader::read(&mut buf).unwrap();

        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(false, ContentNature::None, None, None)]
    #[case(true, ContentNature::None, None, None)]
    #[case(true, ContentNature::Data, None, Some(SerializedData::from_vec(vec![255, 254, 253, 252])))]
    #[case(true, ContentNature::Key, None, Some(SerializedData::from_vec(vec![255, 254])))]
    fn data_sub_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] data_or_key: ContentNature,
        #[case] inline_qos: Option<ParameterList>,
        #[case] serialized_data: Option<SerializedData>,
        injected_checker: Checker,
    ) {
        let expected = Submessage::data(
            big_endian,
            data_or_key,
            READER_ENTITY_ID,
            WRITER_ENTITY_ID,
            SEQUENCE_NUMBER,
            inline_qos,
            serialized_data,
        );

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, SequenceNumberSet::new(SequenceNumber(1234), &[SequenceNumber(1235)]), Count::default())]
    fn acknack_sub_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] writer_sn_state: SequenceNumberSet,
        #[case] count: Count,
        injected_checker: Checker,
    ) {
        let expected = Submessage::acknack(
            big_endian,
            READER_ENTITY_ID,
            WRITER_ENTITY_ID,
            writer_sn_state,
            count,
        );

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, SequenceNumber(1234), SequenceNumberSet::new(SequenceNumber(1234), &[SequenceNumber(1235)]), None, None)]
    fn gap_sub_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] gap_start: SequenceNumber,
        #[case] gap_list: SequenceNumberSet,
        #[case] gap_group_info: Option<GapGroupInfo>,
        #[case] filtered_count: Option<ChangeCount>,
        injected_checker: Checker,
    ) {
        let expected = Submessage::gap(
            big_endian,
            READER_ENTITY_ID,
            WRITER_ENTITY_ID,
            gap_start,
            gap_list,
            gap_group_info,
            filtered_count,
        );

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(
        false,
        false,
        false,
        SequenceNumber(1234),
        SequenceNumber(1235),
        Count::new(9),
        None
    )]
    fn heartbeat_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] is_final: bool,
        #[case] liveliness: bool,
        #[case] first_sn: SequenceNumber,
        #[case] last_sn: SequenceNumber,
        #[case] count: Count,
        #[case] heartbeat_group_info: Option<HeartbeatGroupInfo>,
        injected_checker: Checker,
    ) {
        let expected = Submessage::heartbeat(
            big_endian,
            is_final,
            liveliness,
            READER_ENTITY_ID,
            WRITER_ENTITY_ID,
            first_sn,
            last_sn,
            count,
            heartbeat_group_info,
        );

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, false, SequenceNumber(1234), FragmentNumber(1), 5, 10, 2, None, SerializedData::from_vec(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]))]
    fn data_frag_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] is_key: bool,
        #[case] writer_sn: SequenceNumber,
        #[case] fragment_starting_num: FragmentNumber,
        #[case] fragment_in_submessage: u16,
        #[case] sample_size: u32,
        #[case] fragment_size: u16,
        #[case] inline_qos: Option<ParameterList>,
        #[case] serialized_payload: SerializedData,
        injected_checker: Checker,
    ) {
        let expected = Submessage::datafrag(
            big_endian,
            is_key,
            READER_ENTITY_ID,
            WRITER_ENTITY_ID,
            writer_sn,
            fragment_starting_num,
            fragment_in_submessage,
            fragment_size,
            sample_size,
            inline_qos,
            serialized_payload,
        );

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, SequenceNumber(1234), FragmentNumber(1), Count::new(9))]
    fn heartbeat_frag_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] writer_sn: SequenceNumber,
        #[case] last_fragmentation_num: FragmentNumber,
        #[case] count: Count,
        injected_checker: Checker,
    ) {
        let expected = Submessage::heartbeatfrag(
            big_endian,
            READER_ENTITY_ID,
            WRITER_ENTITY_ID,
            writer_sn,
            last_fragmentation_num,
            count,
        );

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, SequenceNumber(1234), FragmentNumberSet::new(FragmentNumber(1), &[FragmentNumber(2), FragmentNumber(3)]), Count::new(9))]
    fn nack_frag_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] writer_sn: SequenceNumber,
        #[case] fragmentation_number_state: FragmentNumberSet,
        #[case] count: Count,
        injected_checker: Checker,
    ) {
        let expected = Submessage::nackfrag(
            big_endian,
            READER_ENTITY_ID,
            WRITER_ENTITY_ID,
            writer_sn,
            fragmentation_number_state,
            count,
        );

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, None)]
    #[case(false, Some(Timestamp::default()))]
    fn info_timestamp_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] timestamp: Option<Timestamp>,
        injected_checker: Checker,
    ) {
        let expected = Submessage::info_timestamp(big_endian, timestamp);

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, LocatorList::new(vec![Locator::new(LocatorKind::UdpV4, [0u8; 16], 0)]), None)]
    #[case(false, LocatorList::new(vec![Locator::new(LocatorKind::UdpV4, [0u8; 16], 0)]), Some(LocatorList::new(vec![Locator::new(LocatorKind::UdpV4, [0u8; 16], 0)])))]
    fn info_reply_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] unicast_locator_list: LocatorList,
        #[case] multicast_locator_list: Option<LocatorList>,
        injected_checker: Checker,
    ) {
        let expected =
            Submessage::info_reply(big_endian, unicast_locator_list, multicast_locator_list);

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, DUMMY_GUID_PREFIX)]
    fn info_destination_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] guid_prefix: GuidPrefix,
        injected_checker: Checker,
    ) {
        let expected = Submessage::info_destination(big_endian, guid_prefix);

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, ProtocolVersion::V2_4, DUMMY_VENDOR_ID, DUMMY_GUID_PREFIX)]
    fn info_source_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] protocol_version: ProtocolVersion,
        #[case] vendor_id: VendorId,
        #[case] guid_prefix: GuidPrefix,
        injected_checker: Checker,
    ) {
        let expected =
            Submessage::info_source(big_endian, protocol_version, vendor_id, guid_prefix);

        injected_checker.check(expected);
    }

    #[rstest]
    #[case(false, None, None, None, None, None, None)]
    fn header_extension_serialization_deserialization(
        #[case] big_endian: bool,
        #[case] message_length: Option<MessageLength>,
        #[case] rtps_send_timestamp: Option<Timestamp>,
        #[case] u_extension4: Option<UExtension4>,
        #[case] w_extension8: Option<WExtension8>,
        #[case] message_checksum: Option<Checksum32>,
        #[case] parameters: Option<ParameterList>,
        injected_checker: Checker,
    ) {
        let expected = Submessage::header_extension(
            big_endian,
            message_length,
            rtps_send_timestamp,
            u_extension4,
            w_extension8,
            message_checksum,
            parameters,
        );

        injected_checker.check(expected);
    }
}
