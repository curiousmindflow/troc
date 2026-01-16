use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[binrw]
pub struct ParameterId(pub(crate) i16);

impl ParameterId {
    pub const PID_PAD: ParameterId = ParameterId(0x0000);
    pub const PID_SENTINEL: ParameterId = ParameterId(0x0001);
    pub const PID_USER_DATA: ParameterId = ParameterId(0x002c);
    pub const PID_TOPIC_NAME: ParameterId = ParameterId(0x0005);
    pub const PID_TYPE_NAME: ParameterId = ParameterId(0x0007);
    pub const PID_GROUP_DATA: ParameterId = ParameterId(0x002d);
    pub const PID_TOPIC_DATA: ParameterId = ParameterId(0x002e);
    pub const PID_DURABILITY: ParameterId = ParameterId(0x001d);
    pub const PID_DURABILITY_SERVICE: ParameterId = ParameterId(0x001e);
    pub const PID_DEADLINE: ParameterId = ParameterId(0x0023);
    pub const PID_LATENCY_BUDGET: ParameterId = ParameterId(0x0027);
    pub const PID_LIVELINESS: ParameterId = ParameterId(0x001b);
    pub const PID_RELIABILITY: ParameterId = ParameterId(0x001a);
    pub const PID_LIFESPAN: ParameterId = ParameterId(0x002b);
    pub const PID_DESTINATION_ORDER: ParameterId = ParameterId(0x0025);
    pub const PID_HISTORY: ParameterId = ParameterId(0x0040);
    pub const PID_RESOURCE_LIMITS: ParameterId = ParameterId(0x0041);
    pub const PID_OWNERSHIP: ParameterId = ParameterId(0x001f);
    pub const PID_OWNERSHIP_STRENGTH: ParameterId = ParameterId(0x0006);
    pub const PID_PRESENTATION: ParameterId = ParameterId(0x0021);
    pub const PID_PARTITION: ParameterId = ParameterId(0x0029);
    pub const PID_TIME_BASED_FILTER: ParameterId = ParameterId(0x0004);
    pub const PID_TRANSPORT_PRIORITY: ParameterId = ParameterId(0x0049);
    pub const PID_DOMAIN_ID: ParameterId = ParameterId(0x000f);
    pub const PID_DOMAIN_TAG: ParameterId = ParameterId(0x4014);
    pub const PID_PROTOCOL_VERSION: ParameterId = ParameterId(0x0015);
    pub const PID_VENDORID: ParameterId = ParameterId(0x0016);
    pub const PID_UNICAST_LOCATOR: ParameterId = ParameterId(0x002f);
    pub const PID_MULTICAST_LOCATOR: ParameterId = ParameterId(0x0030);
    pub const PID_DEFAULT_UNICAST_LOCATOR: ParameterId = ParameterId(0x0031);
    pub const PID_DEFAULT_MULTICAST_LOCATOR: ParameterId = ParameterId(0x0048);
    pub const PID_METATRAFFIC_UNICAST_LOCATOR: ParameterId = ParameterId(0x0032);
    pub const PID_METATRAFFIC_MULTICAST_LOCATOR: ParameterId = ParameterId(0x0033);
    pub const PID_EXPECTS_INLINE_QOS: ParameterId = ParameterId(0x0043);
    pub const PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT: ParameterId = ParameterId(0x0034);
    pub const PID_PARTICIPANT_LEASE_DURATION: ParameterId = ParameterId(0x0002);
    pub const PID_CONTENT_FILTER_PROPERTY: ParameterId = ParameterId(0x0035);
    pub const PID_PARTICIPANT_GUID: ParameterId = ParameterId(0x0050);
    pub const PID_GROUP_GUID: ParameterId = ParameterId(0x0052);
    pub const PID_GROUP_ENTITY_ID: ParameterId = ParameterId(0x0053);
    pub const PID_BUILTIN_ENDPOINT_SET: ParameterId = ParameterId(0x0058);
    pub const PID_BUILTIN_ENDPOINT_QOS: ParameterId = ParameterId(0x0077);
    pub const PID_PROPERTY_LIST: ParameterId = ParameterId(0x0059);
    pub const PID_TYPE_MAX_SIZE_SERIALIZED: ParameterId = ParameterId(0x0060);
    pub const PID_ENTITY_NAME: ParameterId = ParameterId(0x0062);
    pub const PID_ENDPOINT_GUID: ParameterId = ParameterId(0x005a);

    pub const PID_CONTENT_FILTER_INFO: ParameterId = ParameterId(0x0055);
    pub const PID_COHERENT_SET: ParameterId = ParameterId(0x0056);
    pub const PID_DIRECTED_WRITE: ParameterId = ParameterId(0x0057);
    pub const PID_ORIGINAL_WRITER_INFO: ParameterId = ParameterId(0x0061);
    pub const PID_GROUP_COHERENT_SET: ParameterId = ParameterId(0x0063);
    pub const PID_GROUP_SEQ_NUM: ParameterId = ParameterId(0x0064);
    pub const PID_WRITER_GROUP_INFO: ParameterId = ParameterId(0x0065);
    pub const PID_SECURE_WRITER_GROUP_INFO: ParameterId = ParameterId(0x0066);
    pub const PID_KEY_HASH: ParameterId = ParameterId(0x0070);
    pub const PID_STATUS_INFO: ParameterId = ParameterId(0x0071);

    // TODO: is this official ?
    pub const PID_TIMESTAMP: ParameterId = ParameterId(0x6FFF);
}

impl Display for ParameterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))?;
        Ok(())
    }
}
