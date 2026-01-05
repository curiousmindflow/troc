use std::mem::size_of_val;

use binrw::{binrw, helpers::count};

use crate::types::{
    Timestamp,
    change_count::ChangeCount,
    checksum::Checksum32,
    count::Count,
    fragment_number::FragmentNumber,
    fragment_number_set::FragmentNumberSet,
    groupd_digest::GroupDigest,
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
    submessage_kind::SubmessageKind,
    u_extension4::UExtension4,
    vendor_id::VendorId,
    w_extension8::WExtension8,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[binrw]
#[brw(import(flags: SubmessageFlags, len: u16, id: SubmessageKind, pos: u16))]
pub enum SubmessageContent {
    // 8.3.8.1
    #[br(pre_assert(id == SubmessageKind::AckNack))]
    AckNack {
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn_state: SequenceNumberSet,
        count: Count,
    },
    // 8.3.8.2
    #[br(pre_assert(id == SubmessageKind::Data))]
    Data {
        extra_flags: SubmessageExtraFlags,
        octets_to_inline_qos: u16,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        #[brw(if(flags.second() == 1))]
        inline_qos: Option<ParameterList>,
        #[brw(if(flags.third() == 1 || flags.fourth() == 1))]
        #[br(args(pos + len))]
        serialized_data: Option<SerializedData>,
    },
    // 8.3.8.4
    #[br(pre_assert(id == SubmessageKind::Gap))]
    Gap {
        reader_id: EntityId,
        writer_id: EntityId,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
        #[brw(if(flags.second() == 1))]
        gap_group_info: Option<GapGroupInfo>,
        #[brw(if(flags.third() == 1))]
        filtered_count: Option<ChangeCount>,
    },
    // 8.3.8.6
    // 56
    #[br(pre_assert(id == SubmessageKind::Heartbeat))]
    Heartbeat {
        reader_id: EntityId,
        writer_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        count: Count,
        #[brw(if(flags.fourth() == 1))]
        heartbeat_group_info: Option<HeartbeatGroupInfo>,
    },
    // 8.3.8.3
    #[br(pre_assert(id == SubmessageKind::DataFrag))]
    DataFrag {
        flags: SubmessageFlags,
        extra_flags: SubmessageFlags,
        octets_to_inline_qos: u16,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        fragment_starting_num: FragmentNumber,
        fragment_in_submessage: u16,
        fragment_size: u16,
        sample_size: u32,
        #[brw(if(flags.second() == 1))]
        inline_qos: Option<ParameterList>,
        #[br(args(pos + len))]
        serialized_payload: SerializedData,
    },
    // 8.3.8.7
    // 28 bytes
    #[br(pre_assert(id == SubmessageKind::HeartbeatFrag))]
    HeartbeatFrag {
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        last_fragmentation_num: FragmentNumber,
        count: Count,
    },
    // 8.3.8.12
    #[br(pre_assert(id == SubmessageKind::NackFrag))]
    NackFrag {
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        fragmentation_number_state: FragmentNumberSet,
        count: Count,
    },
    // 8.3.8.11
    #[br(pre_assert(id == SubmessageKind::InfoTs))]
    InfoTimestamp {
        #[brw(if(flags.second() == 0))]
        timestamp: Option<Timestamp>,
    },
    // 8.3.8.9
    #[br(pre_assert(id == SubmessageKind::InfoReply))]
    InfoReply {
        unicast_locator_list: LocatorList,
        #[brw(if(flags.second() == 1))]
        multicast_locator_list: Option<LocatorList>,
    },
    // 8.3.8.8
    #[br(pre_assert(id == SubmessageKind::InfoDst))]
    InfoDestination { guid_prefix: GuidPrefix },
    // 8.3.8.10
    #[br(pre_assert(id == SubmessageKind::InfoSrc))]
    InfoSource {
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
    },
    // 8.3.3.2
    #[br(pre_assert(id == SubmessageKind::RtpsHe))]
    HeaderExtension {
        #[brw(if(flags.second() == 1))]
        message_length: Option<MessageLength>,
        #[brw(if(flags.third() == 1))]
        rtps_send_timestamp: Option<Timestamp>,
        #[brw(if(flags.fourth() == 1))]
        u_extension4: Option<UExtension4>,
        #[brw(if(flags.fifth() == 1))]
        w_extension8: Option<WExtension8>,
        #[brw(if(flags.sixth() != 0 && flags.sixth() != 0))]
        message_checksum: Option<Checksum32>,
        #[brw(if(flags.eighth() == 1))]
        parameters: Option<ParameterList>,
    },
    // 8.3.8.13
    #[br(pre_assert(id == SubmessageKind::Pad))]
    Pad,
    // #[br(pre_assert(id != SubmessageKind::AckNack && id != SubmessageKind::Data && id != SubmessageKind::Gap && id != SubmessageKind::Heartbeat && id != SubmessageKind::DataFrag && id != SubmessageKind::HeartbeatFrag && id != SubmessageKind::NackFrag && id != SubmessageKind::InfoTs && id != SubmessageKind::InfoReply && id != SubmessageKind::InfoDst && id != SubmessageKind::InfoSrc ))]
    #[br(pre_assert(id == SubmessageKind::Unknown))]
    Unknown {
        #[br(count = len)]
        data: Vec<u8>,
    },
}

impl SubmessageContent {
    pub fn size(&self) -> usize {
        match self {
            SubmessageContent::Data {
                extra_flags,
                octets_to_inline_qos,
                reader_id,
                writer_id,
                writer_sn,
                inline_qos,
                serialized_data,
            } => {
                let mut size = size_of_val(extra_flags)
                    + size_of_val(octets_to_inline_qos)
                    + size_of_val(reader_id)
                    + size_of_val(writer_id)
                    + size_of_val(writer_sn);
                if let Some(inner) = inline_qos {
                    size += inner.size();
                }
                if let Some(inner) = serialized_data {
                    size += inner.size();
                }
                size
            }
            SubmessageContent::AckNack {
                reader_id,
                writer_id,
                writer_sn_state,
                count,
            } => {
                let mut size = size_of_val(reader_id);
                size += size_of_val(writer_id);
                size += writer_sn_state.size();
                size += size_of_val(count);
                size
            }
            SubmessageContent::Gap {
                reader_id,
                writer_id,
                gap_start,
                gap_list,
                gap_group_info,
                filtered_count,
            } => {
                let mut size = 0;
                size += size_of_val(reader_id);
                size += size_of_val(writer_id);
                size += size_of_val(gap_start);
                size += gap_list.size();
                if let Some(inner) = gap_group_info {
                    size += size_of_val(inner)
                }
                if let Some(inner) = filtered_count {
                    size += size_of_val(inner)
                }
                size
            }
            SubmessageContent::Heartbeat {
                reader_id,
                writer_id,
                first_sn,
                last_sn,
                count,
                heartbeat_group_info,
            } => {
                let mut size = size_of_val(reader_id)
                    + size_of_val(writer_id)
                    + size_of_val(first_sn)
                    + size_of_val(last_sn)
                    + size_of_val(count);
                if let Some(inner) = heartbeat_group_info {
                    size += size_of_val(inner)
                }
                size
            }
            SubmessageContent::DataFrag {
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
            } => {
                let mut size = 0;
                size += size_of_val(flags);
                size += size_of_val(extra_flags);
                size += size_of_val(octets_to_inline_qos);
                size += size_of_val(reader_id);
                size += size_of_val(writer_id);
                size += size_of_val(writer_sn);
                size += size_of_val(fragment_starting_num);
                size += size_of_val(fragment_in_submessage);
                size += size_of_val(fragment_size);
                size += size_of_val(sample_size);
                size += serialized_payload.size();

                if let Some(inner) = inline_qos {
                    size += inner.size();
                }
                size
            }
            SubmessageContent::NackFrag {
                reader_id,
                writer_id,
                writer_sn,
                fragmentation_number_state,
                count,
            } => {
                let mut size = size_of_val(reader_id)
                    + size_of_val(writer_id)
                    + size_of_val(writer_sn)
                    + size_of_val(count);
                size += fragmentation_number_state.size();
                size
            }
            SubmessageContent::HeartbeatFrag {
                reader_id,
                writer_id,
                writer_sn,
                last_fragmentation_num,
                count,
            } => {
                size_of_val(reader_id)
                    + size_of_val(writer_id)
                    + size_of_val(writer_sn)
                    + size_of_val(last_fragmentation_num)
                    + size_of_val(count)
            }
            SubmessageContent::InfoDestination { guid_prefix } => size_of_val(guid_prefix),
            SubmessageContent::InfoReply {
                unicast_locator_list,
                multicast_locator_list,
            } => {
                let mut size = size_of_val(unicast_locator_list);
                if let Some(inner) = multicast_locator_list {
                    size += size_of_val(inner)
                }
                size
            }
            SubmessageContent::InfoSource {
                protocol_version,
                vendor_id,
                guid_prefix,
            } => size_of_val(protocol_version) + size_of_val(vendor_id) + size_of_val(guid_prefix),
            SubmessageContent::InfoTimestamp { timestamp } => {
                let mut size = 0;
                if let Some(timestamp) = timestamp {
                    size += timestamp.size();
                }
                size
            }
            SubmessageContent::Pad => 0,
            SubmessageContent::HeaderExtension {
                message_length,
                rtps_send_timestamp,
                u_extension4,
                w_extension8,
                message_checksum,
                parameters,
            } => {
                let mut size = 0;
                if let Some(inner) = message_length {
                    size += size_of_val(inner)
                }
                if let Some(inner) = rtps_send_timestamp {
                    size += size_of_val(inner)
                }
                if let Some(inner) = u_extension4 {
                    size += size_of_val(inner)
                }
                if let Some(inner) = w_extension8 {
                    size += size_of_val(inner)
                }
                if let Some(inner) = message_checksum {
                    size += size_of_val(inner)
                }
                if let Some(inner) = parameters {
                    size += size_of_val(inner)
                }
                size
            }
            SubmessageContent::Unknown { data } => data.len(),
        }
    }

    pub fn id(&self) -> SubmessageKind {
        match self {
            SubmessageContent::Data { .. } => SubmessageKind::Data,
            SubmessageContent::AckNack { .. } => SubmessageKind::AckNack,
            SubmessageContent::Gap { .. } => SubmessageKind::Gap,
            SubmessageContent::Heartbeat { .. } => SubmessageKind::Heartbeat,
            SubmessageContent::DataFrag { .. } => SubmessageKind::DataFrag,
            SubmessageContent::NackFrag { .. } => SubmessageKind::NackFrag,
            SubmessageContent::HeartbeatFrag { .. } => SubmessageKind::HeartbeatFrag,
            SubmessageContent::InfoDestination { .. } => SubmessageKind::InfoDst,
            SubmessageContent::InfoReply { .. } => SubmessageKind::InfoReply,
            SubmessageContent::InfoSource { .. } => SubmessageKind::InfoSrc,
            SubmessageContent::InfoTimestamp { .. } => SubmessageKind::InfoTs,
            SubmessageContent::Pad => SubmessageKind::Pad,
            SubmessageContent::HeaderExtension { .. } => SubmessageKind::RtpsHe,
            SubmessageContent::Unknown { .. } => SubmessageKind::Unknown,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[binrw]
pub struct GapGroupInfo {
    gap_start_gsn: SequenceNumber,
    gap_end_gsn: SequenceNumber,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[binrw]
pub struct HeartbeatGroupInfo {
    current_gsn: SequenceNumber,
    first_gsn: SequenceNumber,
    last_gsn: SequenceNumber,
    writer_set: GroupDigest,
    secure_writer_set: GroupDigest,
}
