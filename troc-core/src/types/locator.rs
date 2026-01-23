use std::{
    fmt::{Debug, Display},
    net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use binrw::binrw;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::types::locator_kind::{LOCATOR_KIND_INVALID, LocatorKind};

pub static LOCATOR_PORT_INVALID: u32 = 0;

pub static LOCATOR_ADDRESS_INVALID: [u8; 16] = [0; 16];

pub static LOCATOR_INVALID: Locator = Locator {
    kind: LOCATOR_KIND_INVALID,
    address: LOCATOR_ADDRESS_INVALID,
    port: LOCATOR_PORT_INVALID,
};

#[derive(Clone, Default, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[binrw]
pub struct Locator {
    pub kind: LocatorKind,
    pub port: u32,
    pub address: [u8; 16],
}

impl Locator {
    const REGEX_STRING: &'static str =
        r"(?P<addr>(?:[0-9]{1,3}\.){3}[0-9]{1,3}):(?P<port>\d{1,5}):(?P<kind>(UDPV4|IPC))";

    pub fn new(kind: LocatorKind, address: [u8; 16], port: u32) -> Self {
        Self {
            kind,
            address,
            port,
        }
    }

    fn from_ipv4addr_to_generic_addr(ipv4addr: Ipv4Addr) -> [u8; 16] {
        let mut addr = vec![0u8; 12];
        let mut real = ipv4addr.octets().to_vec();
        addr.append(&mut real);
        addr.try_into().unwrap()
    }
}

impl FromStr for Locator {
    type Err = regex::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(Locator::REGEX_STRING).expect("Bad locator regex");
        if let Some(caps) = re.captures(s) {
            let addr = &caps["addr"];
            let port = &caps["port"];
            let _kind = &caps["kind"];
            let complete = &format!("{addr}:{port}");
            let locator = Locator::from(complete).unwrap();
            Ok(locator)
        } else {
            Err(regex::Error::Syntax("No regex match".to_string()))
        }
    }
}

impl Debug for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Locator")
            .field("kind", &self.kind)
            .field("address", &self.address)
            .field("port", &self.port)
            .finish()
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = match self.kind {
            LocatorKind::UdpV4 => self
                .address
                .iter()
                .skip(12)
                .map(|s| s.to_string())
                .join("."),
            _ => self
                .address
                .iter()
                .map(|segment| segment.to_string())
                .join("."),
        };

        f.write_str(&format!("{}:{}::{}", address, self.port, self.kind,))?;
        Ok(())
    }
}

impl Locator {
    pub fn from(origin: impl AsRef<str>) -> Result<Locator, AddrParseError> {
        let sockaddr: SocketAddr = origin.as_ref().parse()?;
        let locator = match sockaddr.ip() {
            IpAddr::V4(addr) => Locator {
                kind: LocatorKind::UdpV4,
                address: Locator::from_ipv4addr_to_generic_addr(addr),
                port: sockaddr.port().into(),
            },
            _ => unreachable!(),
        };
        Ok(locator)
    }
}

impl TryFrom<SocketAddr> for Locator {
    type Error = ();

    fn try_from(value: SocketAddr) -> Result<Self, Self::Error> {
        match value {
            SocketAddr::V4(sock_addr_v4) => {
                let loc = Self {
                    kind: LocatorKind::UdpV4,
                    address: Locator::from_ipv4addr_to_generic_addr(*sock_addr_v4.ip()),
                    port: sock_addr_v4.port().into(),
                };
                Ok(loc)
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod locator_tests {
    use regex::Regex;
    use rstest::rstest;

    use super::Locator;

    #[rstest]
    #[case("127.0.0.1", "9000", "UDPV4")]
    #[case("0.0.0.0", "65000", "IPC")]
    #[should_panic]
    #[case("0.0.0.0", "65000", "SOMETHING_WRONG")]
    fn from_str_components(#[case] addr: &str, #[case] port: &str, #[case] kind: &str) {
        let complete = &format!("{addr}:{port}:{kind}");

        let re = Regex::new(Locator::REGEX_STRING).unwrap();
        if let Some(caps) = re.captures(complete) {
            assert_eq!(addr, &caps["addr"]);
            assert_eq!(port, &caps["port"]);
            assert_eq!(kind, &caps["kind"]);
        } else {
            panic!()
        }
    }
}
