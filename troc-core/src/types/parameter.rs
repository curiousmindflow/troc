use std::mem::size_of;

use binrw::binrw;
use cdr::{CdrBe, Infinite};
use serde::{Deserialize, Serialize};

use crate::types::Locator;

use super::{
    builtin_endpoint_qos::BuiltinEndpointQos,
    builtin_endpoint_set::BuiltinEndpointSet,
    count::Count,
    domain_id::DomainId,
    duration::Duration,
    guid::{EntityId, Guid},
    parameter_id::ParameterId,
    protocol_version::ProtocolVersion,
    vendor_id::VendorId,
};

pub const PID_PAD_ID: ParameterId = ParameterId(0x0000);
pub const PID_SENTINEL_ID: ParameterId = ParameterId(0x0001);

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[binrw]
pub struct Parameter {
    pub(crate) parameter_id: ParameterId,
    pub(crate) length: i16,
    #[br(count = length as usize)]
    pub(crate) value: Vec<u8>,
}

impl Parameter {
    pub fn new(parameter_id: ParameterId, value: &[u8]) -> Self {
        let length = value.len().try_into().unwrap();
        let value = value.to_vec();
        Self {
            parameter_id,
            length,
            value,
        }
    }

    pub fn size(&self) -> usize {
        let mut size = 0;
        size += size_of::<ParameterId>();
        size += size_of::<u16>();
        size += self.value.len();
        size
    }

    pub fn pid_pad() -> Parameter {
        Parameter::new(PID_PAD_ID, &[])
    }

    pub fn pid_sentinel() -> Parameter {
        Parameter::new(PID_SENTINEL_ID, &[])
    }

    pub fn pid_user_data(user_data_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x002c), user_data_qos_policy)
    }

    pub fn pid_topic_name(topic_name: &str) -> Parameter {
        assert!(topic_name.len() < 256);
        let topic_name = topic_name.as_bytes();
        Parameter::new(ParameterId::PID_TOPIC_NAME, topic_name)
    }

    pub fn pid_type_name(type_name: &str) -> Parameter {
        assert!(type_name.len() < 256);
        let topic_name = type_name.as_bytes();
        Parameter::new(ParameterId(0x0007), topic_name)
    }

    pub fn pid_group_data(group_data_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x002d), group_data_qos_policy)
    }

    pub fn pid_topic_data(topic_data_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x002e), topic_data_qos_policy)
    }

    pub fn pid_durability(durability_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x001d), durability_qos_policy)
    }

    pub fn pid_durability_service(durability_service_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x001e), durability_service_qos_policy)
    }

    pub fn pid_deadline(deadline_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0023), deadline_qos_policy)
    }

    pub fn pid_latency_budget(latency_budget_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0027), latency_budget_qos_policy)
    }

    pub fn pid_liveliness(liveliness_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x001b), liveliness_qos_policy)
    }

    pub fn pid_reliability(reliability_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x001a), reliability_qos_policy)
    }

    pub fn pid_lifespan(lifespan_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x002b), lifespan_qos_policy)
    }

    pub fn pid_destination_order(destination_order_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0025), destination_order_qos_policy)
    }

    pub fn pid_history(history_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0040), history_qos_policy)
    }

    pub fn pid_resource_limits(resource_limits_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0041), resource_limits_qos_policy)
    }

    pub fn pid_ownership(ownership_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x001f), ownership_qos_policy)
    }

    pub fn pid_ownership_strength(ownership_strength_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0006), ownership_strength_qos_policy)
    }

    pub fn pid_presentation(presentation_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0021), presentation_qos_policy)
    }

    pub fn pid_partition(partition_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0029), partition_qos_policy)
    }

    pub fn pid_time_based_filter(time_based_filter_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0004), time_based_filter_qos_policy)
    }

    pub fn pid_transport_priority(transport_priority_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0049), transport_priority_qos_policy)
    }

    pub fn pid_domain_id(domain_id_qos_policy: DomainId) -> Parameter {
        let domain_id_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&domain_id_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x000f), &domain_id_qos_policy)
    }

    pub fn pid_domain_tag(domain_tag_qos_policy: &&str) -> Parameter {
        assert!(domain_tag_qos_policy.len() < 256);
        let domain_tag_qos_policy = domain_tag_qos_policy.as_bytes();
        Parameter::new(ParameterId(0x4014), domain_tag_qos_policy)
    }

    pub fn pid_protocol_version(protocol_version_qos_policy: ProtocolVersion) -> Parameter {
        let protocol_version_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&protocol_version_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0015), &protocol_version_qos_policy)
    }

    pub fn pid_vendor_id(pvendor_id_qos_policy: VendorId) -> Parameter {
        let pvendor_id_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&pvendor_id_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0016), &pvendor_id_qos_policy)
    }

    pub fn pid_unicast_locator(unicast_locator_qos_policy: Locator) -> Parameter {
        let unicast_locator_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&unicast_locator_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x002f), &unicast_locator_qos_policy)
    }

    pub fn pid_multicast_locator(multicast_locator_qos_policy: Locator) -> Parameter {
        let multicast_locator_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&multicast_locator_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0030), &multicast_locator_qos_policy)
    }

    pub fn pid_default_unicast_locator(default_unicast_locator_qos_policy: Locator) -> Parameter {
        let default_unicast_locator_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&default_unicast_locator_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0031), &default_unicast_locator_qos_policy)
    }

    pub fn pid_default_multicast_locator(
        default_multicast_locator_qos_policy: Locator,
    ) -> Parameter {
        let default_multicast_locator_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&default_multicast_locator_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0048), &default_multicast_locator_qos_policy)
    }

    pub fn pid_metatraffic_unicast_locator(
        metatraffic_unicast_locator_qos_policy: Locator,
    ) -> Parameter {
        let metatraffic_unicast_locator_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&metatraffic_unicast_locator_qos_policy, Infinite)
                .unwrap();
        Parameter::new(ParameterId(0x0032), &metatraffic_unicast_locator_qos_policy)
    }

    pub fn pid_metatraffic_multicast_locator(
        metatraffic_multicast_locator_qos_policy: Locator,
    ) -> Parameter {
        let metatraffic_multicast_locator_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&metatraffic_multicast_locator_qos_policy, Infinite)
                .unwrap();
        Parameter::new(
            ParameterId(0x0033),
            &metatraffic_multicast_locator_qos_policy,
        )
    }

    pub fn pid_expects_ineline_qos(expects_ineline_qos_qos_policy: bool) -> Parameter {
        let expects_ineline_qos_qos_policy: u8 = expects_ineline_qos_qos_policy.into();
        Parameter::new(ParameterId(0x0043), &[expects_ineline_qos_qos_policy])
    }

    pub fn pid_participant_manual_liveliness_count(
        participant_manual_liveliness_count_qos_policy: Count,
    ) -> Parameter {
        let participant_manual_liveliness_count_qos_policy = cdr::serialize::<_, _, CdrBe>(
            &participant_manual_liveliness_count_qos_policy,
            Infinite,
        )
        .unwrap();
        Parameter::new(
            ParameterId(0x0034),
            &participant_manual_liveliness_count_qos_policy,
        )
    }

    pub fn pid_participant_lease_duration(
        participant_lease_duration_qos_policy: Duration,
    ) -> Parameter {
        let participant_lease_duration_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&participant_lease_duration_qos_policy, Infinite)
                .unwrap();
        Parameter::new(ParameterId(0x0002), &participant_lease_duration_qos_policy)
    }

    pub fn pid_content_filter_property(content_filter_property_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0035), content_filter_property_qos_policy)
    }

    pub fn pid_participant_guid(participant_guid_qos_policy: Guid) -> Parameter {
        let participant_guid_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&participant_guid_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0050), &participant_guid_qos_policy)
    }

    pub fn pid_group_guid(group_guid_qos_policy: Guid) -> Parameter {
        let group_guid_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&group_guid_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0052), &group_guid_qos_policy)
    }

    pub fn pid_group_entity_id(group_entity_id_qos_policy: EntityId) -> Parameter {
        let group_entity_id_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&group_entity_id_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0053), &group_entity_id_qos_policy)
    }

    pub fn pid_builtin_endpoint_set(
        builtin_endpoint_set_qos_policy: BuiltinEndpointSet,
    ) -> Parameter {
        let builtin_endpoint_set_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&builtin_endpoint_set_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0058), &builtin_endpoint_set_qos_policy)
    }

    pub fn pid_builtin_endpoint_qos(
        builtin_endpoint_qos_qos_policy: BuiltinEndpointQos,
    ) -> Parameter {
        let builtin_endpoint_qos_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&builtin_endpoint_qos_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0077), &builtin_endpoint_qos_qos_policy)
    }

    pub fn pid_property_list(property_list_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0059), property_list_qos_policy)
    }

    pub fn pid_type_max_size_serialized(type_max_size_serialized_qos_policy: u64) -> Parameter {
        let type_max_size_serialized_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&type_max_size_serialized_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x0060), &type_max_size_serialized_qos_policy)
    }

    pub fn pid_entity_name(entity_name_qos_policy: &[u8]) -> Parameter {
        Parameter::new(ParameterId(0x0062), entity_name_qos_policy)
    }

    pub fn pid_endpoint_guid(endpoint_guid_qos_policy: Guid) -> Parameter {
        let endpoint_guid_qos_policy =
            cdr::serialize::<_, _, CdrBe>(&endpoint_guid_qos_policy, Infinite).unwrap();
        Parameter::new(ParameterId(0x005a), &endpoint_guid_qos_policy)
    }
}
