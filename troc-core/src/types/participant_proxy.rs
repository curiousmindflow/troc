use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::{
    Count, DomainTag, ENTITYID_PARTICIPANT, Guid, GuidPrefix, Locator, LocatorList, VendorId,
    builtin_endpoint_qos::BuiltinEndpointQos, builtin_endpoint_set::BuiltinEndpointSet,
    domain_id::DomainId, protocol_version::ProtocolVersion,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParticipantProxy {
    pub domain_id: DomainId,
    pub domain_tag: DomainTag,
    pub protocol_version: ProtocolVersion,
    pub guid_prefix: GuidPrefix,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub available_builtin_endpoints: BuiltinEndpointSet,
    pub builtin_endpoint_qos: BuiltinEndpointQos,
    pub metatraffic_unicast_locator_list: LocatorList,
    pub metatraffic_multicast_locator_list: LocatorList,
    pub default_unicast_locator_list: LocatorList,
    pub default_multicast_locator_list: LocatorList,
    pub manual_liveliness_count: Count,
}

impl ParticipantProxy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: DomainTag,
        expects_inline_qos: bool,
        metatraffic_unicast_locator_list: LocatorList,
        metatraffic_multicast_locator_list: LocatorList,
        default_unicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        available_builtin_endpoints: BuiltinEndpointSet,
        builtin_endpoint_qos: BuiltinEndpointQos,
    ) -> Self {
        Self {
            domain_id,
            domain_tag,
            protocol_version: ProtocolVersion::V2_4,
            guid_prefix,
            vendor_id: VendorId::default(),
            expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            available_builtin_endpoints,
            manual_liveliness_count: Count::default(),
            builtin_endpoint_qos,
        }
    }

    pub fn get_guid(&self) -> Guid {
        Guid::new(self.guid_prefix, ENTITYID_PARTICIPANT)
    }

    pub fn get_guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }

    pub fn get_domain_tag(&self) -> DomainTag {
        self.domain_tag.clone()
    }

    pub fn get_available_builtin_endpoints(&self) -> BuiltinEndpointSet {
        self.available_builtin_endpoints
    }

    pub fn get_metatraffic_unicast_locator_list(&self) -> LocatorList {
        self.metatraffic_unicast_locator_list.clone()
    }

    pub fn get_metatraffic_multicast_locator_list(&self) -> LocatorList {
        self.metatraffic_multicast_locator_list.clone()
    }

    pub fn get_default_unicast_locator_list(&self) -> LocatorList {
        self.default_unicast_locator_list.clone()
    }

    pub fn get_default_multicast_locator_list(&self) -> LocatorList {
        self.default_multicast_locator_list.clone()
    }
}

impl Display for ParticipantProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("GuidPrefix: {}, BuiltinEndpoints: {}, MetaUniLocators: {}, MetaMultiLocators: {}, UniLocators: {}, MultiLocators: {}, DomainId: {}, DomainTag: {:?}, ProtocolVersion: {}, VendorId: {}, ExpectInlineQos: {}, ManualLiveliness: {}, BuiltinEndpointQos: {}", 
        self.guid_prefix, self.available_builtin_endpoints, self.metatraffic_unicast_locator_list, self.metatraffic_multicast_locator_list, self.default_unicast_locator_list, self.default_multicast_locator_list, self.domain_id, self.domain_tag, self.protocol_version, self.vendor_id, self.expects_inline_qos,
        self.manual_liveliness_count, self.builtin_endpoint_qos
    ))?;
        Ok(())
    }
}
