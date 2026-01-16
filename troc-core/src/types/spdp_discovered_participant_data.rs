use std::io::Cursor;

use binrw::{BinRead, Endian};
use cdr::Error;
use serde::{Deserialize, Serialize};

use super::{
    Count, DomainTag, ENTITYID_PARTICIPANT, ENTITYID_UNKOWN, Guid, InstanceHandle, Locator,
    LocatorList, ParameterId, ParameterList, ParticipantProxy, SerializedData, VendorId,
    builtin_endpoint_qos::BuiltinEndpointQos, builtin_endpoint_set::BuiltinEndpointSet,
    builtin_topic_key::BuiltinTopicKey, domain_id::DomainId, duration::Duration,
    participant_builtin_topic_data::ParticipantBuiltinTopicData, protocol_version::ProtocolVersion,
    user_data_qos_policy::UserDataQosPolicy,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PdpDiscoveredParticipantData {
    pub(crate) dds_participant_data: ParticipantBuiltinTopicData,
    pub(crate) participant_proxy: ParticipantProxy,
    pub(crate) lease_duration: Duration,
}

impl PdpDiscoveredParticipantData {
    pub fn new(
        dds_participant_data: ParticipantBuiltinTopicData,
        participant_proxy: ParticipantProxy,
        lease_duration: Duration,
    ) -> Self {
        Self {
            dds_participant_data,
            participant_proxy,
            lease_duration,
        }
    }

    pub fn get_proxy(&self) -> ParticipantProxy {
        self.participant_proxy.clone()
    }

    pub fn get_lease_duration(&self) -> Duration {
        self.lease_duration
    }

    pub fn get_guid(&self) -> Guid {
        Guid::new(self.participant_proxy.get_guid_prefix(), ENTITYID_UNKOWN)
    }

    pub fn from_serialized_data(data: SerializedData) -> Result<Self, Error> {
        let (param_list, endian) = ParameterList::from_serialized_data(data).unwrap();
        let data = Self::from_parameter_list(&param_list, endian);
        Ok(data)
    }

    pub fn into_serialized_data(&self, endian: Endian) -> Result<SerializedData, Error> {
        let parameter_list = Self::into_parameter_list(self, endian);
        let data = parameter_list.into_serialized_data(endian).unwrap();
        Ok(data)
    }

    fn from_parameter_list(parameter_list: &ParameterList, endian: Endian) -> Self {
        let key = parameter_list
            .get_param::<BuiltinTopicKey>(ParameterId::PID_KEY_HASH, endian)
            .unwrap_or_default();
        let user_data = parameter_list
            .get_param::<UserDataQosPolicy>(ParameterId::PID_USER_DATA, endian)
            .unwrap_or_default();
        let domain_id = parameter_list
            .get_param::<DomainId>(ParameterId::PID_DOMAIN_ID, endian)
            .unwrap_or_default();
        let domain_tag = parameter_list
            .get_param::<DomainTag>(ParameterId::PID_DOMAIN_TAG, endian)
            .unwrap_or_default();
        let protocol_version = parameter_list
            .get_param::<ProtocolVersion>(ParameterId::PID_PROTOCOL_VERSION, endian)
            .unwrap_or_default();
        let guid_prefix = parameter_list
            .get_param::<Guid>(ParameterId::PID_PARTICIPANT_GUID, endian)
            .unwrap_or_default()
            .get_guid_prefix();
        let vendor_id = parameter_list
            .get_param::<VendorId>(ParameterId::PID_VENDORID, endian)
            .unwrap_or_default();
        let expects_inline_qos = parameter_list
            .get_param_raw(ParameterId::PID_EXPECTS_INLINE_QOS)
            .map(|v| v[0] != 0)
            .unwrap_or_default();
        let available_builtin_endpoints = parameter_list
            .get_param::<BuiltinEndpointSet>(ParameterId::PID_BUILTIN_ENDPOINT_SET, endian)
            .unwrap_or_default();
        let builtin_endpoint_qos = parameter_list
            .get_param::<BuiltinEndpointQos>(ParameterId::PID_BUILTIN_ENDPOINT_QOS, endian)
            .unwrap_or_default();
        let metatraffic_unicast_locator_list = (&*parameter_list
            .get_params::<Locator>(ParameterId::PID_METATRAFFIC_UNICAST_LOCATOR, endian))
            .into();
        let metatraffic_multicast_locator_list = (&*parameter_list
            .get_params::<Locator>(ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR, endian))
            .into();
        let default_unicast_locator_list = (&*parameter_list
            .get_params::<Locator>(ParameterId::PID_DEFAULT_UNICAST_LOCATOR, endian))
            .into();
        let default_multicast_locator_list = (&*parameter_list
            .get_params::<Locator>(ParameterId::PID_DEFAULT_MULTICAST_LOCATOR, endian))
            .into();
        let manual_liveliness_count = parameter_list
            .get_param::<Count>(ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, endian)
            .unwrap_or_default();
        let lease_duration = parameter_list
            .get_param::<Duration>(ParameterId::PID_PARTICIPANT_LEASE_DURATION, endian)
            .unwrap_or_default();

        Self {
            dds_participant_data: ParticipantBuiltinTopicData { key, user_data },
            participant_proxy: ParticipantProxy {
                domain_id,
                domain_tag,
                protocol_version,
                guid_prefix,
                vendor_id,
                expects_inline_qos,
                available_builtin_endpoints,
                builtin_endpoint_qos,
                metatraffic_unicast_locator_list,
                metatraffic_multicast_locator_list,
                default_unicast_locator_list,
                default_multicast_locator_list,
                manual_liveliness_count,
            },
            lease_duration,
        }
    }

    pub fn into_parameter_list(&self, endian: Endian) -> ParameterList {
        let mut parameter_list = ParameterList::new();

        parameter_list.set_param(
            ParameterId::PID_KEY_HASH,
            self.dds_participant_data.key.clone(),
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_USER_DATA,
            self.dds_participant_data.user_data.clone(),
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_DOMAIN_ID,
            self.participant_proxy.domain_id,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_DOMAIN_TAG,
            self.participant_proxy.domain_tag.clone(),
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_PROTOCOL_VERSION,
            self.participant_proxy.protocol_version,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_PARTICIPANT_GUID,
            Guid::new(self.participant_proxy.guid_prefix, ENTITYID_PARTICIPANT),
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_VENDORID,
            self.participant_proxy.vendor_id,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_EXPECTS_INLINE_QOS,
            self.participant_proxy.expects_inline_qos as u8,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_BUILTIN_ENDPOINT_SET,
            self.participant_proxy.available_builtin_endpoints,
            endian,
        );
        // parameter_list.set_param(
        //     ParameterId::PID_BUILTIN_ENDPOINT_QOS,
        //     self.participant_proxy.builtin_endpoint_qos,
        //     endian,
        // );

        for locator in self
            .participant_proxy
            .metatraffic_unicast_locator_list
            .iter()
        {
            parameter_list.set_param(
                ParameterId::PID_METATRAFFIC_UNICAST_LOCATOR,
                locator,
                endian,
            );
        }

        for locator in self
            .participant_proxy
            .metatraffic_multicast_locator_list
            .iter()
        {
            parameter_list.set_param(
                ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR,
                locator,
                endian,
            );
        }

        for locator in self.participant_proxy.default_unicast_locator_list.iter() {
            parameter_list.set_param(ParameterId::PID_DEFAULT_UNICAST_LOCATOR, locator, endian);
        }

        for locator in self.participant_proxy.default_multicast_locator_list.iter() {
            parameter_list.set_param(ParameterId::PID_DEFAULT_MULTICAST_LOCATOR, locator, endian);
        }

        parameter_list.set_param(
            ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            self.participant_proxy.manual_liveliness_count,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_PARTICIPANT_LEASE_DURATION,
            self.lease_duration,
            endian,
        );

        parameter_list
    }
}
