use std::{net::Ipv4Addr, ops::Range, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};

use troc_core::{DomainTag, Locator, LocatorList, domain_id::DomainId};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub global: GlobalConfiguration,
    #[serde(default)]
    pub reader: ReaderConfiguration,
    #[serde(default)]
    pub writer: WriterConfiguration,
    #[serde(default)]
    pub discovery: DiscoveryConfiguration,
}

impl Configuration {
    pub fn get_global_default_unicast_address(&self) -> Vec<Ipv4Addr> {
        self.global
            .default_unicast_adress
            .iter()
            .map(|a| <Ipv4Addr as FromStr>::from_str(a).unwrap())
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default = "self::GlobalConfiguration::default")]
pub struct GlobalConfiguration {
    pub domain_id: DomainId,
    pub domain_tag: DomainTag,
    pub expects_inline_qos: bool,
    pub unicast_port_range: Range<u16>,
    pub default_multicast_locator_list: LocatorList,
    pub data_max_size_serialized: u32,
    pub fragment_size: u16,
    //
    pub default_unicast_adress: Vec<String>,
    pub default_multicast_address: String,
    pub port_base: u32,
    pub domain_gain: u32,
    pub participant_gain: u32,
    pub d0: u32,
    pub d1: u32,
    pub d2: u32,
    pub d3: u32,
}

impl Default for GlobalConfiguration {
    fn default() -> Self {
        Self {
            domain_id: DomainId(0),
            domain_tag: DomainTag::default(),
            expects_inline_qos: false,
            unicast_port_range: 7000..u16::MAX,
            default_multicast_locator_list: LocatorList::new(vec![
                <Locator as FromStr>::from_str("224.0.0.5:9005:UDPV4").unwrap(),
            ]),
            data_max_size_serialized: 60 * 1024,
            fragment_size: 60 * 1024,
            //
            default_unicast_adress: vec![String::from("127.0.0.1")],
            default_multicast_address: String::from("239.255.0.1"),
            port_base: 7400,
            domain_gain: 250,
            participant_gain: 2,
            d0: 0,
            d1: 10,
            d2: 1,
            d3: 11,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default = "self::ReaderConfiguration::default")]
pub struct ReaderConfiguration {
    pub heartbeat_response_delay: Duration,
    pub heartbeat_suppression_delay: Duration,
}

impl Default for ReaderConfiguration {
    fn default() -> Self {
        Self {
            heartbeat_response_delay: Duration::from_millis(200),
            heartbeat_suppression_delay: Duration::from_millis(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default = "self::WriterConfiguration::default")]
pub struct WriterConfiguration {
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration: Duration,
    pub piggyback_heartbeat: bool,
    pub piggyback_timestamp: bool,
}

impl Default for WriterConfiguration {
    fn default() -> Self {
        Self {
            heartbeat_period: Duration::from_millis(250),
            nack_response_delay: Duration::from_millis(200),
            nack_suppression_duration: Duration::from_millis(0),
            piggyback_heartbeat: true,
            piggyback_timestamp: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default = "self::DiscoveryConfiguration::default")]
pub struct DiscoveryConfiguration {
    pub liveliness_locator_list: LocatorList,
    pub participant_discovery_locator_list: LocatorList,
    pub endpoint_publication_discovery_locator_list: LocatorList,
    pub endpoint_subscription_discovery_locator_list: LocatorList,
    pub announcement_period: Duration,
    pub lease_duration: Duration,
}

impl Default for DiscoveryConfiguration {
    fn default() -> Self {
        Self {
            participant_discovery_locator_list: LocatorList::new(vec![
                <Locator as FromStr>::from_str("224.0.0.1:9000:UDPV4").unwrap(),
            ]),
            endpoint_publication_discovery_locator_list: LocatorList::new(vec![
                <Locator as FromStr>::from_str("224.0.0.2:9001:UDPV4").unwrap(),
            ]),
            endpoint_subscription_discovery_locator_list: LocatorList::new(vec![
                <Locator as FromStr>::from_str("224.0.0.3:9002:UDPV4").unwrap(),
            ]),
            liveliness_locator_list: LocatorList::new(vec![
                <Locator as FromStr>::from_str("224.0.0.4:9003:UDPV4").unwrap(),
            ]),
            announcement_period: Duration::from_secs(5),
            lease_duration: Duration::from_secs(30),
        }
    }
}
