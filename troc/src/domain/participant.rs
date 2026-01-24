use std::collections::HashMap;

use kameo::Actor;
use kameo::actor::{ActorRef, Spawn};
use kameo::prelude::Message;
use tokio::sync::broadcast::{Receiver, channel};
use troc_core::builtin_endpoint_qos::BuiltinEndpointQos;
use troc_core::domain_id::DomainId;
use troc_core::{DdsError, DiscoveryConfiguration, Locator};
use troc_core::{DomainTag, EntityId, EntityKey};
use troc_core::{
    ENTITYID_PARTICIPANT, Guid, ParticipantProxy, TopicKind, VENDORID_UNKNOWN,
    builtin_endpoint_set::BuiltinEndpointSet,
};

use crate::ParticipantEvent;
use crate::discovery::{DiscoveryActor, DiscoveryActorCreateObject};
use crate::publication::{Publisher, PublisherActor, PublisherActorCreateObject};
use crate::subscription::{Subscriber, SubscriberActor, SubscriberActorCreateObject};
use crate::time::TimerActor;
use crate::wires::{
    ReceiverWireActor, ReceiverWireFactoryActorMessage, SenderWireActor,
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
    guid: Option<Guid>,
    domain_id: Option<u32>,
    configuration: Option<Configuration>,
}

impl DomainParticipantBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_guid(mut self, guid: Guid) -> Self {
        self.guid = Some(guid);
        self
    }

    pub fn with_domain(mut self, domain_id: u32) -> Self {
        self.domain_id = Some(domain_id);
        self
    }

    pub fn with_config(mut self, config: Configuration) -> Self {
        self.configuration = Some(config);
        self
    }

    pub async fn build(self) -> DomainParticipant {
        let DomainParticipantBuilder {
            guid,
            domain_id,
            configuration,
        } = self;
        let configuration =
            configuration.unwrap_or_else(|| Self::retrieve_configuration(None).unwrap());
        let guid = guid.unwrap_or(Guid::generate(VENDORID_UNKNOWN, ENTITYID_PARTICIPANT));
        let domain_id = domain_id.unwrap_or_default();

        let actor = DomainParticipantActor::spawn(DomainParticipantActorCreationObject {
            guid,
            domain_id,
            configuration: configuration.clone(),
        });
        actor.wait_for_startup().await;

        DomainParticipant { guid, actor }
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
        self._event_receiver.take().unwrap()
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
    guid: Guid,
    _infos: ParticipantProxy,
    config: Configuration,
    timer: ActorRef<TimerActor>,
    wire_factory: ActorRef<WireFactoryActor>,
    discovery: ActorRef<DiscoveryActor>,
    entity_identifier: ActorRef<EntityIdentifierActor>,
    publishers: Vec<ActorRef<PublisherActor>>,
    subscribers: Vec<ActorRef<SubscriberActor>>,
    _event_receiver: Option<Receiver<ParticipantEvent>>,
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
        let (event_sender, event_receiver) = channel(64);

        let mut input_wires = Vec::default();
        let (receiver_many_to_many, receiver_locators_many_to_many) = wire_factory
            .ask(ReceiverWireFactoryActorMessage::SPDP)
            .await
            .unwrap();
        input_wires.extend(receiver_many_to_many);
        let (receiver_on_to_one, receiver_locators_on_to_one) = wire_factory
            .ask(ReceiverWireFactoryActorMessage::SEDP)
            .await
            .unwrap();
        input_wires.extend(receiver_on_to_one);
        let locators = receiver_locators_many_to_many
            .clone()
            .merge(receiver_locators_on_to_one.clone());

        let input_wires: HashMap<Locator, ActorRef<ReceiverWireActor>> =
            HashMap::from_iter(locators.iter().zip(input_wires).map(|(a, b)| (*a, b)));

        let mut output_wires = Vec::default();
        let (sender_many_to_many, sender_locators_many_to_many) = wire_factory
            .ask(SenderWireFactoryActorMessage::SPDP)
            .await
            .unwrap();
        output_wires.extend(sender_many_to_many);

        let output_wires: HashMap<Locator, ActorRef<SenderWireActor>> = HashMap::from_iter(
            sender_locators_many_to_many
                .iter()
                .zip(output_wires)
                .map(|(a, b)| (*a, b)),
        );

        let metatraffic_unicast_locator_list = receiver_locators_on_to_one;
        let mut metatraffic_multicast_locator_list =
            receiver_locators_many_to_many.merge(sender_locators_many_to_many);
        metatraffic_multicast_locator_list.sort();
        metatraffic_multicast_locator_list.dedup();

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

        let discovery_configuration = DiscoveryConfiguration {
            announcement_period: args.configuration.discovery.announcement_period.as_millis()
                as i64,
            lease_duration: args.configuration.discovery.lease_duration,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
        };

        let discovery = DiscoveryActor::spawn(DiscoveryActorCreateObject {
            participant_proxy: infos.clone(),
            discovery_configuration,
            input_wires,
            output_wires,
            event_sender,
            timer: timer.clone(),
            wire_factory: wire_factory.clone(),
        });

        discovery.wait_for_startup().await;
        actor_ref.link(&discovery).await;

        let domain_participant_actor = Self {
            guid: args.guid,
            _infos: infos,
            config: args.configuration,
            timer,
            wire_factory,
            discovery,
            entity_identifier,
            publishers: Vec::default(),
            subscribers: Vec::default(),
            _event_receiver: Some(event_receiver),
        };
        Ok(domain_participant_actor)
    }

    async fn on_stop(
        &mut self,
        _actor_ref: kameo::prelude::WeakActorRef<Self>,
        _reason: kameo::prelude::ActorStopReason,
    ) -> Result<(), Self::Error> {
        for publisher in self.publishers.iter() {
            publisher.stop_gracefully().await.unwrap();
            publisher.wait_for_shutdown().await;
        }
        for subscriber in self.subscribers.iter() {
            subscriber.stop_gracefully().await.unwrap();
            subscriber.wait_for_shutdown().await;
        }
        self.discovery.stop_gracefully().await.unwrap();
        self.discovery.wait_for_shutdown().await;
        self.timer.stop_gracefully().await.unwrap();
        self.timer.wait_for_shutdown().await;
        self.wire_factory.stop_gracefully().await.unwrap();
        self.wire_factory.wait_for_shutdown().await;
        Ok(())
    }

    async fn on_link_died(
        &mut self,
        _actor_ref: kameo::prelude::WeakActorRef<Self>,
        _id: kameo::prelude::ActorId,
        _reason: kameo::prelude::ActorStopReason,
    ) -> Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error> {
        // relaunch dead Actor if they didn't die on purpose
        panic!()
    }
}
