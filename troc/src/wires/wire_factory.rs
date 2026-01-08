use super::{Wire, WireError, WireList, udpv4_wire::UdpV4Wire};
use crate::{domain::Configuration, domain::UdpHelper};
use bytes::BytesMut;
use kameo::{Actor, Reply, actor::ActorRef, prelude::Message};
use local_ip_address::local_ip;
use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::Arc,
};
use troc_core::types::{Locator, LocatorKind, LocatorList};

pub type WireFactoryResult = Result<Vec<ActorRef<WireActor>>, WireError>;

#[derive(Debug)]
pub enum WireFactoryActorMessage {
    CreateOutputWiresFromLocators { locators: Vec<Locator> },
    CreateLocalhostInputWires,
    CreateInputWires,
    CreateInputWiresFromLocators { locators: Vec<Locator> },
}

impl Message<WireFactoryActorMessage> for WireFactoryActor {
    type Reply = WireFactoryResult;

    async fn handle(
        &mut self,
        msg: WireFactoryActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            WireFactoryActorMessage::CreateOutputWiresFromLocators { locators } => todo!(),
            WireFactoryActorMessage::CreateLocalhostInputWires => {
                let localhost_locator: Locator = "127.0.0.1".parse().unwrap();
                todo!()
            }
            WireFactoryActorMessage::CreateInputWires => todo!(),
            WireFactoryActorMessage::CreateInputWiresFromLocators { locators } => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WireFactoryActor {
    domain_id: Arc<u32>,
    config: Arc<Configuration>,
}

impl Actor for WireFactoryActor {
    type Args = Self;

    type Error = WireError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl WireFactoryActor {
    pub fn new(domain_id: u32, config: Configuration) -> Self {
        let domain_id = Arc::new(domain_id);
        let config = Arc::new(config);

        Self { domain_id, config }
    }

    pub fn build_sender_wirelist_from_locators(
        &self,
        locators: &LocatorList,
    ) -> Result<WireList, WireError> {
        let mut wires = Vec::new();
        for locator in locators.iter() {
            if let Ok(wire) = self.build_sender_wire_from_locator(locator) {
                wires.push(wire);
            }
        }
        Ok(WireList::new(wires))
    }

    pub fn build_listener_wirelist_from_locators(
        &self,
        locators: &LocatorList,
    ) -> Result<WireList, WireError> {
        let mut wires = Vec::new();
        for locator in locators.iter() {
            if let Ok(wire) = self.build_listener_wire_from_locator(locator) {
                wires.push(wire);
            }
        }
        Ok(WireList::new(wires))
    }

    pub fn build_sender_wire_from_locator(&self, locator: &Locator) -> Result<Wire, WireError> {
        match locator.kind {
            LocatorKind::UdpV4 => {
                let wired = UdpV4Wire::new_sender(locator, true)?;
                let wire = Wire::new(Box::new(wired));
                Ok(wire)
            }
            _ => Err(WireError::CreationError(std::io::Error::new(
                ErrorKind::InvalidInput,
                "UdpV6 not supported",
            ))),
        }
    }

    pub fn build_listener_wire_from_locator(&self, locator: &Locator) -> Result<Wire, WireError> {
        match locator.kind {
            LocatorKind::UdpV4 => {
                let wired = UdpV4Wire::new_listener(locator, true)?;
                let wire = Wire::new(Box::new(wired));
                Ok(wire)
            }
            _ => Err(WireError::CreationError(std::io::Error::new(
                ErrorKind::InvalidInput,
                "UdpV6 not supported",
            ))),
        }
    }

    pub fn build_user_wire(&self, ip: Option<Ipv4Addr>) -> Result<Wire, WireError> {
        let ip = if let Some(ip) = ip {
            ip
        } else {
            let IpAddr::V4(ip) = local_ip().unwrap() else {
                panic!()
            };
            ip
        };

        let address = UdpHelper::from_ipv4addr_to_generic_addr(ip);
        let mut port = self.generate_user_unicast_base_port() as u16;

        let wired = loop {
            let locator = Locator::new(LocatorKind::UdpV4, address, port as u32);
            if let Ok(wire) = UdpV4Wire::new_listener(&locator, false) {
                break wire;
            }
            if port == u16::MAX {
                panic!("all port has been exhausted");
            }
            port += 1;
        };

        let wire = Wire::new(Box::new(wired));

        Ok(wire)
    }

    pub fn build_discovery_sender_multicast_wire(&self) -> Result<Wire, WireError> {
        let discovery_multicast_address =
            Ipv4Addr::from_str(&self.config.global.default_multicast_address).unwrap();
        let discovery_multicast_generic_address =
            UdpHelper::from_ipv4addr_to_generic_addr(discovery_multicast_address);
        let discovery_multicast_port = self.generate_discovery_multicast_port();
        let locator = Locator::new(
            LocatorKind::UdpV4,
            discovery_multicast_generic_address,
            discovery_multicast_port,
        );

        match locator.kind {
            LocatorKind::UdpV4 => {
                let wired = UdpV4Wire::new_sender(&locator, true)?;
                let wire = Wire::new(Box::new(wired));
                Ok(wire)
            }
            _ => unimplemented!(),
        }
    }

    pub fn build_discovery_listener_multicast_wire(&self) -> Result<Wire, WireError> {
        let discovery_multicast_address =
            Ipv4Addr::from_str(&self.config.global.default_multicast_address).unwrap();
        let discovery_multicast_generic_address =
            UdpHelper::from_ipv4addr_to_generic_addr(discovery_multicast_address);
        let discovery_multicast_port = self.generate_discovery_multicast_port();
        let locator = Locator::new(
            LocatorKind::UdpV4,
            discovery_multicast_generic_address,
            discovery_multicast_port,
        );

        match locator.kind {
            LocatorKind::UdpV4 => {
                let wired = UdpV4Wire::new_listener(&locator, true)?;
                let wire = Wire::new(Box::new(wired));
                Ok(wire)
            }
            _ => unimplemented!(),
        }
    }

    pub fn build_discovery_unicast_wire(&self) -> Result<Wire, WireError> {
        let IpAddr::V4(ip) = local_ip().unwrap() else {
            panic!()
        };
        let address = UdpHelper::from_ipv4addr_to_generic_addr(ip);
        let mut port = self.generate_discovery_unicast_port() as u16;

        let wired = loop {
            let locator = Locator::new(LocatorKind::UdpV4, address, port as u32);
            if let Ok(wire) = UdpV4Wire::new_listener(&locator, false) {
                break wire;
            }
            if port == u16::MAX {
                panic!("all port has been exhausted");
            }
            port += 1;
        };

        let wire = Wire::new(Box::new(wired));

        Ok(wire)
    }

    fn generate_discovery_multicast_port(&self) -> u32 {
        let global_conf = &self.config.global;
        global_conf.port_base + global_conf.domain_gain * *self.domain_id + global_conf.d0
    }

    fn generate_discovery_unicast_port(&self) -> u32 {
        let global_conf = &self.config.global;
        global_conf.port_base
            + global_conf.domain_gain * *self.domain_id
            + global_conf.d1
            + global_conf.participant_gain
    }

    fn generate_user_multicast_port(&self) -> u32 {
        let global_conf = &self.config.global;
        global_conf.port_base + global_conf.domain_gain * *self.domain_id + global_conf.d2
    }

    fn generate_user_unicast_base_port(&self) -> u32 {
        let global_conf = &self.config.global;
        global_conf.port_base
            + global_conf.domain_gain * *self.domain_id
            + global_conf.d3
            + global_conf.participant_gain
    }
}

#[derive(Debug, Reply)]
pub enum WireActorReply {}

#[derive(Debug)]
pub enum WireActorMessage {
    Send(BytesMut),
    Receive,
}

impl Message<WireActorMessage> for WireActor {
    type Reply = WireActorReply;

    async fn handle(
        &mut self,
        msg: WireActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        todo!()
    }
}

#[derive(Debug)]
pub struct WireActor {
    //
}

impl Actor for WireActor {
    type Args = Self;

    type Error = WireError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}
