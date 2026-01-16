use super::{Wire, WireError, WireList, udpv4_wire::UdpV4Wire};
use crate::{domain::Configuration, domain::UdpHelper};
use bytes::BytesMut;
use kameo::{
    Actor,
    actor::{ActorRef, Spawn},
    prelude::Message,
};
use local_ip_address::local_ip;
use std::{
    io::ErrorKind,
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::Arc,
};
use troc_core::{Locator, LocatorKind, LocatorList};

pub trait Sendable: Actor + Sized {
    type Msg: Send + 'static;
    fn build_message(buffer: BytesMut) -> Self::Msg;
}

#[derive(Debug)]
pub enum SenderWireFactoryActorMessage {
    FromLocators { locators: LocatorList },
    SPDP,
}

impl Message<SenderWireFactoryActorMessage> for WireFactoryActor {
    type Reply = (Vec<ActorRef<SenderWireActor>>, LocatorList);

    async fn handle(
        &mut self,
        msg: SenderWireFactoryActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            SenderWireFactoryActorMessage::FromLocators { locators } => todo!(),
            SenderWireFactoryActorMessage::SPDP => {
                let wire = self.build_discovery_sender_multicast_wire().unwrap();
                let locator = wire.locator();
                let sender_wire_actor = SenderWireActor::spawn(SenderWireActor { wire });
                (vec![sender_wire_actor], LocatorList::new(vec![locator]))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReceiverWireFactoryActorMessageDestKind {
    Applicative,
    SPDP,
    SEDP,
}

#[derive()]
pub struct ReceiverWireFactoryActorMessage<T>
where
    T: Actor,
{
    dest: ReceiverWireFactoryActorMessageDestKind,
    actor: ActorRef<T>,
    _phantom: PhantomData<T>,
}

impl<T> ReceiverWireFactoryActorMessage<T>
where
    T: Actor,
{
    pub fn new(dest: ReceiverWireFactoryActorMessageDestKind, actor: ActorRef<T>) -> Self {
        Self {
            dest,
            actor,
            _phantom: PhantomData,
        }
    }
}

impl<T> Message<ReceiverWireFactoryActorMessage<T>> for WireFactoryActor
where
    T: Sendable + Message<T::Msg>,
{
    type Reply = (Vec<ActorRef<ReceiverWireActor<T>>>, LocatorList);

    async fn handle(
        &mut self,
        msg: ReceiverWireFactoryActorMessage<T>,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let ReceiverWireFactoryActorMessage {
            dest,
            actor,
            _phantom,
        } = msg;
        match dest {
            ReceiverWireFactoryActorMessageDestKind::Applicative => todo!(),
            ReceiverWireFactoryActorMessageDestKind::SPDP => {
                let wire = self.build_discovery_listener_multicast_wire().unwrap();
                let locator = wire.locator();
                let receiver_wire_actor = ReceiverWireActor::spawn((actor, wire));
                (vec![receiver_wire_actor], LocatorList::new(vec![locator]))
            }
            ReceiverWireFactoryActorMessageDestKind::SEDP => todo!(),
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
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(args)
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

#[derive(Debug)]
pub struct SenderWireActorMessage {
    pub buffer: BytesMut,
}

impl Message<SenderWireActorMessage> for SenderWireActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: SenderWireActorMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.wire.send(msg.buffer).await.unwrap()
    }
}

#[derive(Debug)]
pub struct SenderWireActor {
    wire: Wire,
}

impl Actor for SenderWireActor {
    type Args = Self;

    type Error = WireError;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(args)
    }
}

#[derive(Debug)]
pub struct ReceiverWireActor<T> {
    _phantom: PhantomData<T>,
}

impl<T> Actor for ReceiverWireActor<T>
where
    T: Sendable + Message<T::Msg>,
{
    type Args = (ActorRef<T>, Wire);

    type Error = WireError;

    async fn on_start(
        mut wire: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        tokio::spawn(async move {
            while let Ok(msg) = wire.1.recv().await {
                wire.0.tell(T::build_message(msg)).await.unwrap();
            }
        });
        Ok(ReceiverWireActor::<T> {
            _phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use kameo::{
        Actor,
        actor::{ActorRef, Spawn},
        error::Infallible,
        prelude::Message,
    };
    use tokio::net::UdpSocket;

    trait Sendable: Actor + Sized {
        type Msg: Send + 'static;
        fn build_message() -> Self::Msg;
    }

    #[derive(Debug)]
    pub struct ReceiveActor<T> {
        _phantom: PhantomData<T>,
    }

    impl<T> Actor for ReceiveActor<T>
    where
        T: Sendable + Message<T::Msg>,
    {
        type Args = ActorRef<T>;

        type Error = Infallible;

        async fn on_start(
            args: Self::Args,
            _actor_ref: kameo::prelude::ActorRef<Self>,
        ) -> Result<Self, Self::Error> {
            let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                while let Ok(_nb_bytes) = socket.recv(&mut buf).await {
                    args.tell(T::build_message()).await.unwrap();
                }
            });
            Ok(ReceiveActor::<T> {
                _phantom: PhantomData,
            })
        }
    }

    #[derive(Debug)]
    pub struct SenderActor {}

    impl Actor for SenderActor {
        type Args = ();

        type Error = Infallible;

        async fn on_start(
            args: Self::Args,
            _actor_ref: kameo::prelude::ActorRef<Self>,
        ) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    #[derive(Debug)]
    pub struct AlphaMessage {}

    impl Message<AlphaMessage> for Alpha {
        type Reply = ();

        async fn handle(
            &mut self,
            msg: AlphaMessage,
            ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
        ) -> Self::Reply {
            todo!()
        }
    }

    #[derive(Debug, Actor)]
    pub struct Alpha {
        //
    }

    impl Sendable for Alpha {
        type Msg = AlphaMessage;

        fn build_message() -> Self::Msg {
            AlphaMessage {}
        }
    }

    #[derive(Debug)]
    pub struct BetaMessage {}

    impl Message<BetaMessage> for Beta {
        type Reply = ();

        async fn handle(
            &mut self,
            msg: BetaMessage,
            ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
        ) -> Self::Reply {
            todo!()
        }
    }

    #[derive(Debug, Actor)]
    pub struct Beta {
        //
    }

    impl Sendable for Beta {
        type Msg = BetaMessage;

        fn build_message() -> Self::Msg {
            BetaMessage {}
        }
    }

    #[derive(Debug)]
    pub struct FactoryCreateSenderMsg {}

    impl Message<FactoryCreateSenderMsg> for Factory {
        type Reply = ActorRef<SenderActor>;

        async fn handle(
            &mut self,
            msg: FactoryCreateSenderMsg,
            ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
        ) -> Self::Reply {
            SenderActor::spawn(())
        }
    }

    #[derive(Debug)]
    pub struct FactoryCreateReceiverMsg<T> {
        _phantom: PhantomData<T>,
    }

    impl<T> FactoryCreateReceiverMsg<T> {
        pub fn new() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    impl<T> Message<FactoryCreateReceiverMsg<T>> for Factory
    where
        T: Sendable + Message<T::Msg>,
    {
        type Reply = ActorRef<T>;

        async fn handle(
            &mut self,
            msg: FactoryCreateReceiverMsg<T>,
            ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
        ) -> Self::Reply {
            todo!()
        }
    }

    #[derive(Debug, Actor)]
    pub struct Factory {
        //
    }

    #[tokio::test]
    async fn main() {
        let factory = Factory::spawn(Factory {});
        let sender_actor = factory.ask(FactoryCreateSenderMsg {}).await.unwrap();
        let receiver_actor = factory
            .ask(FactoryCreateReceiverMsg::<Alpha>::new())
            .await
            .unwrap();
    }
}
