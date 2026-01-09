#![allow(unused_imports)]

pub mod builtin_endpoint_qos;
pub mod builtin_endpoint_set;
pub mod builtin_topic_key;
pub mod change_count;
pub mod change_for_reader_status_kind;
pub mod change_from_writer_status_kind;
pub mod change_kind;
pub mod checksum;
pub mod content_filter_info;
pub mod content_filter_property;
pub mod count;
mod deadline_qos;
mod destination_order_qos;
pub mod domain_id;
mod domain_tag;
mod durability_qos;
pub mod duration;
mod duration_kind;
pub mod entity_name;
pub mod filter_result;
pub mod filter_signature;
pub mod filter_signature_sequence;
pub mod fragment_number;
pub mod fragment_number_set;
pub mod groupd_digest;
pub mod guid;
mod history_qos;
mod inline_qos;
pub mod instance_handle;
pub mod key_hash;
pub mod key_hash_prefix;
pub mod key_hash_suffix;
mod lifespan_qos;
mod liveliness_qos;
pub mod locator;
pub mod locator_kind;
pub mod locator_list;
pub mod message_length;
pub mod original_writer_info;
pub mod parameter;
pub mod parameter_id;
pub mod parameter_list;
pub mod participant_builtin_topic_data;
pub mod participant_message_data;
mod participant_proxy;
pub mod property;
pub mod protocol_id;
pub mod protocol_version;
pub mod reliability_kind;
mod reliability_qos;
mod resource_limits_qos;
pub mod sequence_number;
pub mod sequence_number_set;
pub mod serialized_data;
mod spdp_discovered_participant_data;
pub mod status_info;
mod string;
pub mod submessage_extra_flags;
pub mod submessage_flags;
pub mod submessage_info;
pub mod submessage_kind;
pub mod time;
mod time_based_filter_qos;
mod timestamp;
pub mod topic_kind;
pub mod u_extension4;
pub mod user_data_qos_policy;
pub mod vendor_id;
pub mod w_extension8;

pub use change_count::ChangeCount;
pub use change_kind::ChangeKind;
pub use count::Count;
pub use deadline_qos::DeadlineQosPolicy;
pub use destination_order_qos::DestinationOrderQosPolicy;

pub use domain_tag::DomainTag;
pub use durability_qos::DurabilityQosPolicy;
pub use duration_kind::DurationKind;
pub use fragment_number::FragmentNumber;
pub use fragment_number_set::FragmentNumberSet;
pub use guid::*;
pub use history_qos::HistoryQosPolicy;
pub use inline_qos::InlineQos;
pub use instance_handle::InstanceHandle;
pub use lifespan_qos::LifespanQosPolicy;
pub use liveliness_qos::{LivelinessKind, LivelinessQosPolicy};
pub use locator::Locator;
pub use locator_kind::*;
pub use locator_list::LocatorList;
pub use parameter::Parameter;
pub use parameter_id::ParameterId;
pub use parameter_list::ParameterList;
pub use participant_proxy::ParticipantProxy;
pub use reliability_kind::ReliabilityKind;
pub use reliability_qos::ReliabilityQosPolicy;
pub use resource_limits_qos::ResourceLimitsQosPolicy;
pub use sequence_number::{SEQUENCENUMBER_INVALID, SequenceNumber};
pub use sequence_number_set::SequenceNumberSet;
pub use serialized_data::SerializedData;
pub use spdp_discovered_participant_data::PdpDiscoveredParticipantData;
pub use string::RtpsString;
pub use submessage_flags::SubmessageFlags;
pub use time_based_filter_qos::TimeBasedFilterQosPolicy;
pub use timestamp::{TIME_INFINITE, TIME_INVALID, TIME_ZERO, Timestamp};
pub use topic_kind::TopicKind;
pub use vendor_id::*;

#[derive(Debug, Clone, Copy)]
pub enum ContentNature {
    Data,
    Key,
    None,
}

pub const fn is_target_little_endian() -> bool {
    u16::from_ne_bytes([1, 0]) == 1
}
