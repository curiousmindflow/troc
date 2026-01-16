use std::mem::size_of_val;

use chrono::Utc;

use crate::types::{
    EntityId, SubmessageFlags, TIME_INVALID, Timestamp,
    checksum::{CHECKSUM32_INVALID, Checksum32},
    guid::{GUIDPREFIX_UNKWOWN, GuidPrefix},
    locator::{LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID, Locator},
    locator_kind::LOCATOR_KIND_UDPV4,
    message_length::{MESSAGE_LENGTH_INVALID, MessageLength},
    parameter_id::ParameterId,
    protocol_version::ProtocolVersion,
    submessage_kind::SubmessageKind,
    vendor_id::{VENDORID_UNKNOWN, VendorId},
};

use super::{Submessage, SubmessageContent, message::Message};

// 8.3.4
#[derive(Debug, Default, Clone)]
pub struct MessageReceiver {
    pub source_version: ProtocolVersion,
    pub source_vendor_id: VendorId,
    pub source_guid_prefix: GuidPrefix,
    pub dest_guid_prefix: GuidPrefix,
    pub unicast_reply_locator_list: Vec<Locator>,
    pub multicast_replay_locator_list: Vec<Locator>,
    pub have_timestamp: bool,
    pub timestamp: Option<Timestamp>,
    pub message_length: MessageLength,
    pub message_checksum: Checksum32,
    pub rtps_send_timestamp: Option<Timestamp>,
    pub rtps_reception_timestamp: Timestamp,
    pub clock_skew_detected: bool,
    pub parameters: Vec<ParameterId>,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub flags: SubmessageFlags,
}

impl MessageReceiver {
    pub fn new(dest_guid_prefix: GuidPrefix) -> Self {
        let mut receiver = Self {
            dest_guid_prefix,
            ..Default::default()
        };
        receiver.reset();
        receiver
    }

    pub fn reset(&mut self) {
        self.source_version = ProtocolVersion::V2_4;
        self.source_vendor_id = VENDORID_UNKNOWN;
        self.source_guid_prefix = GUIDPREFIX_UNKWOWN;
        self.unicast_reply_locator_list = vec![Locator::new(
            LOCATOR_KIND_UDPV4,
            LOCATOR_ADDRESS_INVALID,
            LOCATOR_PORT_INVALID,
        )];
        self.multicast_replay_locator_list = vec![Locator::new(
            LOCATOR_KIND_UDPV4,
            LOCATOR_ADDRESS_INVALID,
            LOCATOR_PORT_INVALID,
        )];
        self.have_timestamp = false;
        self.timestamp = None;
        self.message_length = MESSAGE_LENGTH_INVALID;
        self.message_checksum = CHECKSUM32_INVALID;
        self.rtps_send_timestamp = None;
        self.rtps_reception_timestamp = TIME_INVALID.into();
        self.clock_skew_detected = false;
        self.parameters = vec![];
        self.flags = SubmessageFlags::default();
    }

    pub fn capture_header(&mut self, message: &Message) {
        self.source_version = message.header.version;
        self.source_vendor_id = message.header.vendor_id;
        self.source_guid_prefix = message.header.guid_prefix;
        self.message_length = MessageLength(size_of_val(message).try_into().unwrap());
        self.rtps_reception_timestamp = Timestamp::from_datetime(Utc::now());
    }

    pub fn capture_submessage_infos(&mut self, sub_message: &Submessage) {
        self.flags = sub_message.header.flags;
    }

    pub fn capture_entity_ids(&mut self, reader_id: EntityId, writer_id: EntityId) {
        self.reader_id = reader_id;
        self.writer_id = writer_id;
    }
}
