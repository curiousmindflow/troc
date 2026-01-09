use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use troc_core::{locator::Locator, locator_kind::LocatorKind};

pub struct UdpHelper {}

impl UdpHelper {
    pub fn get_socket_addr(locator: &Locator) -> SocketAddr {
        match locator.kind {
            LocatorKind::UdpV4 => {
                let addr: [u8; 4] = locator.address[12..].try_into().unwrap();
                let addr = Ipv4Addr::from(addr);
                let port: u16 = locator.port.try_into().unwrap();

                let sock_addr: SocketAddr = SocketAddr::new(IpAddr::V4(addr), port);
                sock_addr
            }
            _ => unreachable!(),
        }
    }

    pub fn from_ipv4addr_to_generic_addr(ipv4addr: Ipv4Addr) -> [u8; 16] {
        let mut addr = vec![0u8; 12];
        let mut real = ipv4addr.octets().to_vec();
        addr.append(&mut real);
        addr.try_into().unwrap()
    }
}
