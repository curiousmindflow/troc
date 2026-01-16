use std::collections::HashMap;

use kameo::Actor;
use kameo::actor::{ActorRef, Spawn};
use kameo::prelude::Message;
use tokio::sync::broadcast::{Receiver, Sender, channel};
use tracing::{Level, event};
use troc_core::builtin_endpoint_qos::BuiltinEndpointQos;
use troc_core::domain_id::DomainId;
use troc_core::{DdsError, DiscoveryBuilder, DiscoveryConfiguration};
use troc_core::{DomainTag, EntityId, EntityKey};
use troc_core::{
    ENTITYID_PARTICIPANT, Guid, ParticipantProxy, TopicKind, VENDORID_UNKNOWN,
    builtin_endpoint_set::BuiltinEndpointSet,
};

use crate::ParticipantEvent;
use crate::discovery::{DiscoveryActor, DiscoveryActorCreateObject, DiscoveryActorMessage};
use crate::publication::{Publisher, PublisherActor, PublisherActorCreateObject};
use crate::subscription::{Subscriber, SubscriberActor, SubscriberActorCreateObject};
use crate::time::TimerActor;
use crate::wires::{
    ReceiverWireFactoryActorMessage, ReceiverWireFactoryActorMessageDestKind,
    SenderWireFactoryActorMessage, WireFactoryActor,
};
use crate::{
    domain::{
        configuration::Configuration,
        entity_identifier::{EntityIdentifierActor, EntityIdentifierActorAskMessage},
        participant_listener::DomainParticipantListener,
    },
    infrastructure::{QosPolicy, QosPolicyBuilder},
    topic::Topic,
};

#[derive(Default)]
pub struct DomainParticipantBuilder {
    domain_id: u32,
    configuration: Option<Configuration>,
}

impl DomainParticipantBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_domain(mut self, domain_id: u32) -> Self {
        self.domain_id = domain_id;
        self
    }

    pub fn with_config(mut self, config: Configuration) -> Self {
        self.configuration = Some(config);
        self
    }

    pub async fn build(self) -> DomainParticipant {
        let DomainParticipantBuilder {
            domain_id,
            configuration,
        } = self;
        let configuration =
            configuration.unwrap_or_else(|| Self::retrieve_configuration(None).unwrap());
        let guid = Guid::generate(VENDORID_UNKNOWN, ENTITYID_PARTICIPANT);

        let actor = DomainParticipantActor::spawn(DomainParticipantActorCreationObject {
            domain_id,
            guid,
            configuration: configuration.clone(),
        });
        actor.wait_for_startup().await;

        DomainParticipant {
            guid,
            configuration,
            actor,
        }
    }

    fn retrieve_configuration(
        configuration: Option<Configuration>,
    ) -> Result<Configuration, DdsError> {
        let configuration = configuration.unwrap_or_else(|| {
            envious::Config::default()
                .with_prefix("TROC__RTPS__")
                .case_sensitive(false)
                .build_from_env::<Configuration>()
                .unwrap()
            // if let Some(conf_file_path) = configuration.node.conf_file {
            //     ConfiguratonFetcher::fetch(&conf_file_path).unwrap()
            // } else {
            //     configuration
            // }
        });
        Ok(configuration)
    }
}

#[derive(Debug)]
pub struct DomainParticipant {
    guid: Guid,
    configuration: Configuration,
    actor: ActorRef<DomainParticipantActor>,
}

impl DomainParticipant {
    pub fn get_guid(&self) -> Guid {
        self.guid
    }

    // FIXME: should not be necessary
    pub async fn get_participant_proxy(&self) -> ParticipantProxy {
        // self.inner.lock().await.infos.lock().await.clone()
        todo!()
    }

    pub async fn get_listener(&self) -> Result<DomainParticipantListener, DdsError> {
        let receiver = self
            .actor
            .ask(DomainParticipantListenerCreate)
            .await
            .unwrap();
        Ok(DomainParticipantListener { receiver })
    }

    pub fn create_qos_builder(&self) -> QosPolicyBuilder {
        QosPolicyBuilder::new()
    }

    pub fn create_topic<T>(
        &self,
        topic_name: impl AsRef<str>,
        type_name: impl AsRef<str>,
        qos: &QosPolicy,
        topic_kind: TopicKind,
    ) -> Topic<T> {
        Topic::new(topic_name, type_name, qos, topic_kind)
    }

    pub async fn create_publisher(&mut self, qos: &QosPolicy) -> Result<Publisher, DdsError> {
        let publisher = self
            .actor
            .ask(DomainParticipantActorCreatePublisherMessage { qos: *qos })
            .await
            .unwrap();
        Ok(publisher)
    }

    pub async fn create_subscriber(&mut self, qos: &QosPolicy) -> Result<Subscriber, DdsError> {
        let subscriber = self
            .actor
            .ask(DomainParticipantActorCreateSubscriberMessage { qos: *qos })
            .await
            .unwrap();
        Ok(subscriber)
    }
}

#[derive(Debug)]
pub struct DomainParticipantListenerCreate;

impl Message<DomainParticipantListenerCreate> for DomainParticipantActor {
    type Reply = Receiver<ParticipantEvent>;

    async fn handle(
        &mut self,
        _msg: DomainParticipantListenerCreate,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self._event_sender.subscribe()
    }
}

#[derive(Debug)]
struct DomainParticipantActorCreatePublisherMessage {
    qos: QosPolicy,
}

impl Message<DomainParticipantActorCreatePublisherMessage> for DomainParticipantActor {
    type Reply = Result<Publisher, DdsError>;

    async fn handle(
        &mut self,
        msg: DomainParticipantActorCreatePublisherMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.create_publisher(&msg.qos).await
    }
}

#[derive(Debug)]
struct DomainParticipantActorCreateSubscriberMessage {
    qos: QosPolicy,
}

impl Message<DomainParticipantActorCreateSubscriberMessage> for DomainParticipantActor {
    type Reply = Result<Subscriber, DdsError>;

    async fn handle(
        &mut self,
        msg: DomainParticipantActorCreateSubscriberMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.create_subscriber(&msg.qos).await
    }
}

#[derive(Debug)]
struct DomainParticipantActorCreationObject {
    domain_id: u32,
    guid: Guid,
    configuration: Configuration,
}

#[derive(Debug)]
struct DomainParticipantActor {
    domain_id: u32,
    guid: Guid,
    // listener: DomainParticipantListenerHandle,
    infos: ParticipantProxy,
    config: Configuration,
    timer: ActorRef<TimerActor>,
    wire_factory: ActorRef<WireFactoryActor>,
    discovery: ActorRef<DiscoveryActor>,
    entity_identifier: ActorRef<EntityIdentifierActor>,
    publishers: Vec<ActorRef<PublisherActor>>,
    subscribers: Vec<ActorRef<SubscriberActor>>,
    _event_sender: Sender<ParticipantEvent>,
}

impl DomainParticipantActor {
    async fn create_publisher(&mut self, qos: &QosPolicy) -> Result<Publisher, DdsError> {
        let pub_key: EntityKey = self
            .entity_identifier
            .ask(EntityIdentifierActorAskMessage::AskPublisherId)
            .await
            .unwrap()
            .into();
        let pub_id = EntityId::writer_group_builtin(pub_key.0);
        let pub_guid = Guid::new(self.guid.get_guid_prefix(), pub_id);

        let default_unicast_locators = Default::default();
        let default_multicast_locators = self.config.global.default_multicast_locator_list.clone();

        let publisher_actor = PublisherActor::spawn(PublisherActorCreateObject {
            discovery: self.discovery.clone(),
        });

        let publisher = Publisher::new(
            pub_guid,
            default_unicast_locators,
            default_multicast_locators,
            *qos,
            self.config.clone(),
            publisher_actor,
            self.wire_factory.clone(),
            self.discovery.clone(),
            self.entity_identifier.clone(),
            self.timer.clone(),
        );

        Ok(publisher)
    }

    async fn create_subscriber(&mut self, qos: &QosPolicy) -> Result<Subscriber, DdsError> {
        let sub_key: EntityKey = self
            .entity_identifier
            .ask(EntityIdentifierActorAskMessage::AskSubscriberId)
            .await
            .unwrap()
            .into();
        let sub_id = EntityId::writer_group_builtin(sub_key.0);
        let sub_guid = Guid::new(self.guid.get_guid_prefix(), sub_id);

        let default_unicast_locators = Default::default();
        let default_multicast_locators = self.config.global.default_multicast_locator_list.clone();

        let subscriber_actor = SubscriberActor::spawn(SubscriberActorCreateObject {
            discovery: self.discovery.clone(),
        });

        let subscriber = Subscriber::new(
            sub_guid,
            default_unicast_locators,
            default_multicast_locators,
            *qos,
            self.config.clone(),
            subscriber_actor,
            self.wire_factory.clone(),
            self.discovery.clone(),
            self.entity_identifier.clone(),
            self.timer.clone(),
        );

        Ok(subscriber)
    }
}

impl Actor for DomainParticipantActor {
    type Args = DomainParticipantActorCreationObject;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let timer = TimerActor::spawn(TimerActor::new());
        timer.wait_for_startup().await;
        actor_ref.link(&timer).await;
        let wire_factory = WireFactoryActor::spawn(WireFactoryActor::new(
            args.domain_id,
            args.configuration.clone(),
        ));
        wire_factory.wait_for_startup().await;
        actor_ref.link(&wire_factory).await;
        let (event_sender, _) = channel(64);
        let discovery_configuration = DiscoveryConfiguration::new();
        let discovery =
            DiscoveryBuilder::new(args.guid.get_guid_prefix(), discovery_configuration).build();

        let discovery = DiscoveryActor::spawn(DiscoveryActorCreateObject {
            discovery,
            event_sender: event_sender.clone(),
            timer: timer.clone(),
        });
        discovery.wait_for_startup().await;
        actor_ref.link(&discovery).await;
        let entity_identifier = EntityIdentifierActor::spawn(());
        entity_identifier.wait_for_startup().await;
        actor_ref.link(&entity_identifier).await;

        let mut endpoint_set = BuiltinEndpointSet::new();
        endpoint_set.set_disc_builtin_endpoint_participant_announcer(1);
        endpoint_set.set_disc_builtin_endpoint_participant_detector(1);
        // FIXME: when troc node contains only a Reader and rustdds/dustdds contains a writer, if those lines are commented out, the communication doesn't works
        endpoint_set.set_disc_builtin_endpoint_publications_announcer(1);
        endpoint_set.set_disc_builtin_endpoint_publications_detector(1);
        endpoint_set.set_disc_builtin_endpoint_subscriptions_announcer(1);
        endpoint_set.set_disc_builtin_endpoint_subscriptions_detector(1);
        endpoint_set.set_builtin_endpoint_participant_message_data_reader(1);
        endpoint_set.set_builtin_endpoint_participant_message_data_writer(1);

        let mut input_wires = Vec::default();
        let (receiver_many_to_many, receiver_locators_many_to_many) = wire_factory
            .ask(ReceiverWireFactoryActorMessage::<DiscoveryActor>::new(
                ReceiverWireFactoryActorMessageDestKind::SPDP,
                discovery.clone(),
            ))
            .await
            .unwrap();
        input_wires.extend(receiver_many_to_many);
        let (receiver_on_to_one, receiver_locators_on_to_one) = wire_factory
            .ask(ReceiverWireFactoryActorMessage::<DiscoveryActor>::new(
                ReceiverWireFactoryActorMessageDestKind::SEDP,
                discovery.clone(),
            ))
            .await
            .unwrap();
        input_wires.extend(receiver_on_to_one);

        let mut output_wires = Vec::default();
        let (sender_many_to_many, sender_locators_many_to_many) = wire_factory
            .ask(SenderWireFactoryActorMessage::SPDP)
            .await
            .unwrap();
        output_wires.extend(sender_many_to_many);

        let output_wires = HashMap::from_iter(
            sender_locators_many_to_many
                .iter()
                .zip(output_wires)
                .map(|(a, b)| (*a, b)),
        );

        let metatraffic_unicast_locator_list = receiver_locators_on_to_one;
        let metatraffic_multicast_locator_list =
            receiver_locators_many_to_many.merge(sender_locators_many_to_many);

        let infos = ParticipantProxy::new(
            args.guid.get_guid_prefix(),
            DomainId(args.domain_id),
            DomainTag::default(),
            false,
            metatraffic_unicast_locator_list.clone(),
            metatraffic_multicast_locator_list.clone(),
            Default::default(),
            Default::default(),
            endpoint_set,
            BuiltinEndpointQos::default(),
        );

        discovery
            .ask(DiscoveryActorMessage::AddInputWire { wires: input_wires })
            .await
            .unwrap();

        discovery
            .ask(DiscoveryActorMessage::AddOutputWires {
                wires: output_wires,
            })
            .await
            .unwrap();

        discovery
            .tell(DiscoveryActorMessage::ParticipantProxyChanged(
                infos.clone(),
            ))
            .await
            .unwrap();

        let domain_participant_actor = Self {
            domain_id: args.domain_id,
            guid: args.guid,
            // listener: todo!(),
            infos,
            config: args.configuration,
            timer,
            wire_factory,
            discovery,
            entity_identifier,
            publishers: Vec::default(),
            subscribers: Vec::default(),
            _event_sender: event_sender,
        };
        Ok(domain_participant_actor)
    }

    async fn on_stop(
        &mut self,
        actor_ref: kameo::prelude::WeakActorRef<Self>,
        reason: kameo::prelude::ActorStopReason,
    ) -> Result<(), Self::Error> {
        // stop all publisher and subscriber
        // stop DiscoveryActor
        // stop WireFactoryActor
        Ok(())
    }

    async fn on_link_died(
        &mut self,
        actor_ref: kameo::prelude::WeakActorRef<Self>,
        id: kameo::prelude::ActorId,
        reason: kameo::prelude::ActorStopReason,
    ) -> Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error> {
        // relaunch dead Actor if they didn't die on purpose
        panic!()
    }
}

#[cfg(test)]
mod tests {
    use kameo::{
        Actor,
        actor::{ActorRef, Spawn},
        error::Infallible,
        prelude::Message,
    };
    use rstest::rstest;

    #[derive(Debug)]
    struct ParentActor {
        child: Option<ActorRef<ChildActor>>,
    }

    impl ParentActor {
        async fn start_child(parent: &ActorRef<Self>) -> ActorRef<ChildActor> {
            let child = ChildActor::spawn(ChildActor);
            child.wait_for_startup().await;
            child.register("child_actor").unwrap();
            parent.link(&child).await;
            child
        }
    }

    impl Actor for ParentActor {
        type Args = Self;

        type Error = Infallible;

        async fn on_start(
            args: Self::Args,
            actor_ref: kameo::prelude::ActorRef<Self>,
        ) -> Result<Self, Self::Error> {
            let id = actor_ref.id();
            println!("ParentActor::on_start, id: {id}");
            let child = Self::start_child(&actor_ref).await;
            let parent = Self { child: Some(child) };
            Ok(parent)
        }

        async fn on_link_died(
            &mut self,
            actor_ref: kameo::prelude::WeakActorRef<Self>,
            id: kameo::prelude::ActorId,
            reason: kameo::prelude::ActorStopReason,
        ) -> Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error> {
            println!("ParentActor::on_link_died, id: {id}, because of: {reason}");
            let parent = actor_ref.upgrade().unwrap();
            let child = Self::start_child(&parent).await;
            self.child.replace(child);
            Ok(std::ops::ControlFlow::Continue(()))
        }

        async fn on_stop(
            &mut self,
            actor_ref: kameo::prelude::WeakActorRef<Self>,
            reason: kameo::prelude::ActorStopReason,
        ) -> Result<(), Self::Error> {
            let child = self.child.as_ref().unwrap();
            if child.stop_gracefully().await.is_ok() {
                child.wait_for_shutdown().await;
            }
            println!("ParentActor::on_stop, because of: {reason}");
            Ok(())
        }
    }

    #[derive(Debug)]
    struct ChildActor;

    impl Actor for ChildActor {
        type Args = Self;

        type Error = Infallible;

        async fn on_start(
            args: Self::Args,
            actor_ref: kameo::prelude::ActorRef<Self>,
        ) -> Result<Self, Self::Error> {
            let id = actor_ref.id();
            println!("ChildActor::on_start, id: {id}");
            Ok(args)
        }

        async fn on_panic(
            &mut self,
            actor_ref: kameo::prelude::WeakActorRef<Self>,
            err: kameo::prelude::PanicError,
        ) -> Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error> {
            println!("ChildActor::on_panic, because of: {err}");
            Ok(std::ops::ControlFlow::Break(
                kameo::prelude::ActorStopReason::Panicked(err),
            ))
        }

        async fn on_stop(
            &mut self,
            actor_ref: kameo::prelude::WeakActorRef<Self>,
            reason: kameo::prelude::ActorStopReason,
        ) -> Result<(), Self::Error> {
            println!("ChildActor::on_stop, because of: {reason}");
            Ok(())
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum ChildActorMessage {
        Do,
        Crash,
    }

    impl Message<ChildActorMessage> for ChildActor {
        type Reply = ();

        async fn handle(
            &mut self,
            msg: ChildActorMessage,
            ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
        ) -> Self::Reply {
            match msg {
                ChildActorMessage::Do => println!("ChildActor::handle, msg: {msg:?}"),
                ChildActorMessage::Crash => panic!(),
            }
        }
    }

    #[rstest]
    #[case::doo(ChildActorMessage::Do)]
    #[case::crash(ChildActorMessage::Crash)]
    #[tokio::test]
    async fn child_do(#[case] msg: ChildActorMessage) {
        // let dp =
        //     DomainParticipantBuilder::new(0, Configuration::default(), "dp".to_string(), false)
        //         .await
        //         .build();

        let parent = ParentActor::spawn(ParentActor { child: None });
        parent.wait_for_startup().await;

        let child_ref = ActorRef::<ChildActor>::lookup("child_actor")
            .unwrap()
            .unwrap();

        child_ref.tell(msg).await.unwrap();
        child_ref.tell(ChildActorMessage::Do).await.unwrap();

        parent.stop_gracefully().await.unwrap();
        parent.wait_for_shutdown().await;
    }
}
