use kameo::Actor;
use kameo::actor::{ActorRef, Spawn};
use kameo::prelude::Message;
use tracing::{Level, event};
use troc_core::DdsError;
use troc_core::types::builtin_endpoint_qos::BuiltinEndpointQos;
use troc_core::types::domain_id::DomainId;
use troc_core::types::{DomainTag, EntityId, EntityKey};
use troc_core::types::{
    ENTITYID_PARTICIPANT, Guid, LocatorList, ParticipantProxy, TopicKind, VENDORID_UNKNOWN,
    builtin_endpoint_set::BuiltinEndpointSet,
};

use crate::discovery::DiscoveryActor;
use crate::domain::{
    DISCOVERY_ACTOR_NAME, ENTITY_IDENTIFIER_ACTOR_NAME, TIMER_ACTOR_NAME, WIRE_FACTORY_ACTOR_NAME,
};
use crate::publication::{Publisher, PublisherActor, PublisherActorCreateObject};
use crate::time::TimerActor;
use crate::wires::WireFactoryActor;
use crate::{
    domain::{
        configuration::Configuration,
        entity_identifier::{
            AskedId, EntityIdentifierActor, EntityIdentifierActorAskMessage,
            EntityIdentifierActorFreeMessage,
        },
        participant_listener::{DomainParticipantListener, DomainParticipantListenerHandle},
    },
    infrastructure::{QosPolicy, QosPolicyBuilder},
    topic::Topic,
};

#[derive(Default)]
pub struct DomainParticipantBuilder {
    domain_id: u32,
    name: Option<String>,
    listener: bool,
    configuration: Option<Configuration>,
}

impl DomainParticipantBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    pub fn with_listener(mut self) -> Self {
        self.listener = true;
        self
    }

    pub fn with_config(mut self, config: Configuration) -> Self {
        self.configuration = Some(config);
        self
    }

    // async fn new(domain_id: u32, config: Configuration, name: &str, listener: bool) -> Self {
    //     // TODO: choose a vendorid for troc_core
    //     let guid = Guid::generate(VENDORID_UNKNOWN, ENTITYID_PARTICIPANT);
    //     let wire_factory = WireFactory::new(domain_id, config.clone());

    //     // let span = Span::current();
    //     // span.record("guid_prefix", guid.get_guid_prefix().to_string());

    //     let (listener, listener_receiver) = if listener {
    //         let (sender, receiver) = tokio::sync::broadcast::channel(16);
    //         (
    //             DomainParticipantListenerHandle::new(16, Some(sender)),
    //             Some(receiver),
    //         )
    //     } else {
    //         (DomainParticipantListenerHandle::new(0, None), None)
    //     };

    //     let (participant_infos_sender, participant_infos_receiver) = channel(64);

    //     let spdp_writer_wire_list = wire_factory
    //         .build_discovery_sender_multicast_wire()
    //         .unwrap();
    //     let spdp_writer_wire_list = WireList::new(vec![spdp_writer_wire_list]);

    //     let spdp_reader_wire_list = wire_factory
    //         .build_discovery_listener_multicast_wire()
    //         .unwrap();
    //     let spdp_reader_wire_list = WireList::new(vec![spdp_reader_wire_list]);

    //     let sedp_unicast_wire_0 = wire_factory.build_discovery_unicast_wire().unwrap();
    //     let sedp_unicast_wire_1 = wire_factory.build_discovery_unicast_wire().unwrap();
    //     let sedp_unicast_wire_2 = wire_factory.build_discovery_unicast_wire().unwrap();
    //     let sedp_unicast_wire_3 = wire_factory.build_discovery_unicast_wire().unwrap();
    //     let sedp_unicast_wire_list = WireList::new(vec![
    //         sedp_unicast_wire_0,
    //         sedp_unicast_wire_1,
    //         sedp_unicast_wire_2,
    //         sedp_unicast_wire_3,
    //     ]);
    //     // let sedp_multicast_wire_list = wire_factory
    //     //     .build_discovery_listener_multicast_wire()
    //     //     .unwrap();
    //     // let sedp_multicast_wire_list = WireList::new(vec![sedp_multicast_wire_list]);
    //     let sedp_multicast_wire_list = WireList::default();

    //     let metatraffic_unicast_locator_list = sedp_unicast_wire_list.extract_locators();
    //     // let metatraffic_multicast_locator_list = spdp_reader_wire_list.extract_locators();
    //     let metatraffic_multicast_locator_list = LocatorList::default();

    //     let mut endpoint_set = BuiltinEndpointSet::new();
    //     endpoint_set.set_disc_builtin_endpoint_participant_announcer(1);
    //     endpoint_set.set_disc_builtin_endpoint_participant_detector(1);
    //     // FIXME: when troc node contains only a Reader and rustdds/dustdds contains a writer, if those lines are commented out, the communication doesn't works
    //     endpoint_set.set_disc_builtin_endpoint_publications_announcer(1);
    //     endpoint_set.set_disc_builtin_endpoint_publications_detector(1);
    //     endpoint_set.set_disc_builtin_endpoint_subscriptions_announcer(1);
    //     endpoint_set.set_disc_builtin_endpoint_subscriptions_detector(1);
    //     endpoint_set.set_builtin_endpoint_participant_message_data_reader(1);
    //     endpoint_set.set_builtin_endpoint_participant_message_data_writer(1);
    //     //

    //     let inner = DomainParticipantInner::new(
    //         guid,
    //         metatraffic_unicast_locator_list,
    //         metatraffic_multicast_locator_list,
    //         LocatorList::default(),
    //         LocatorList::default(),
    //         endpoint_set,
    //         BuiltinEndpointQos::default(),
    //         disc_cmd_sender,
    //         discovery_notifier.clone(),
    //         cancellation_token.clone(),
    //         listener_receiver,
    //         config.clone(),
    //         span.clone(),
    //     )
    //     .await;

    //     let discovery_handler = DiscoveryHandler::new(
    //         guid,
    //         weak_inner,
    //         Self::agreement,
    //         wire_factory.clone(),
    //         spdp_writer_wire_list,
    //         spdp_reader_wire_list,
    //         sedp_unicast_wire_list,
    //         sedp_multicast_wire_list,
    //         disc_cmd_receiver,
    //         listener.clone(),
    //         config,
    //         cancellation_token.clone(),
    //         name,
    //     )
    //     .await;

    //     event!(Level::INFO, "Participant created");

    //     Self {
    //         domain_id,
    //         guid,
    //         inner,
    //         listener,
    //         discovery_handler,
    //         wire_factory,
    //         span,
    //     }

    //     todo!()
    // }

    pub fn build(self) -> DomainParticipant {
        let DomainParticipantBuilder {
            domain_id,
            name,
            listener,
            configuration,
        } = self;
        // TODO
        let configuration =
            configuration.unwrap_or_else(|| Self::retrieve_configuration(None).unwrap());
        let guid = Guid::generate(VENDORID_UNKNOWN, ENTITYID_PARTICIPANT);

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

        // TODO: find the locators from the configuration
        let metatraffic_unicast_locator_list = Default::default();
        let metatraffic_multicast_locator_list = Default::default();
        let default_unicast_locator_list = Default::default();
        let default_multicast_locator_list = Default::default();

        let infos = ParticipantProxy::new(
            guid.get_guid_prefix(),
            DomainId(domain_id),
            DomainTag::default(),
            false,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            endpoint_set,
            BuiltinEndpointQos::default(),
        );

        let actor = DomainParticipantActor::spawn(DomainParticipantActorCreationObject {
            domain_id,
            guid,
            infos,
            configuration: configuration.clone(),
        });

        DomainParticipant {
            domain_id,
            guid,
            configuration,
            actor,
        }
    }

    fn retrieve_configuration(
        configuration: Option<Configuration>,
    ) -> Result<Configuration, DdsError> {
        let configuration = configuration.unwrap_or_else(|| {
            let configuration = envious::Config::default()
                .with_prefix("TROC__RTPS__")
                .case_sensitive(false)
                .build_from_env::<Configuration>()
                .unwrap();
            configuration
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
    domain_id: u32,
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

    pub async fn get_listener(&self) -> Option<DomainParticipantListener> {
        // if self.listener.sender.is_some() {
        //     self.inner
        //         .lock()
        //         .await
        //         .listener_receiver
        //         .take()
        //         .map(DomainParticipantListener::new)
        // } else {
        //     None
        // }

        todo!()
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
        let entity_identifier =
            ActorRef::<EntityIdentifierActor>::lookup(ENTITY_IDENTIFIER_ACTOR_NAME)
                .unwrap()
                .unwrap();
        let pub_key: EntityKey = entity_identifier
            .ask(EntityIdentifierActorAskMessage::AskPublisherId)
            .await
            .unwrap()
            .into();
        let pub_id = EntityId::writer_group_builtin(pub_key.0);
        let pub_guid = Guid::new(self.guid.get_guid_prefix(), pub_id);

        let default_unicast_locators = Default::default();
        let default_multicast_locators = self
            .configuration
            .global
            .default_multicast_locator_list
            .clone();

        let publisher_actor = PublisherActor::spawn(PublisherActorCreateObject {});

        let publisher = Publisher::new(
            pub_guid,
            default_unicast_locators,
            default_multicast_locators,
            *qos,
            self.configuration.clone(),
            publisher_actor,
        );

        Ok(publisher)
    }

    // pub async fn create_subscriber(&mut self, qos: &QosPolicy) -> Result<Subscriber, DdsError> {
    //     todo!()
    // }
}

#[derive(Debug)]
enum DomainParticipantMessage {
    CreatePublisher,
    CreateSubscriber,
}

impl Message<DomainParticipantMessage> for DomainParticipantActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: DomainParticipantMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            DomainParticipantMessage::CreatePublisher => {
                // TODO
                // call Self::create_publisher(..)
            }
            DomainParticipantMessage::CreateSubscriber => {
                // TODO
                // call Self::create_subscriber(..)
            }
        }
    }
}

#[derive(Debug)]
struct DomainParticipantActorCreationObject {
    domain_id: u32,
    guid: Guid,
    infos: ParticipantProxy,
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
        timer.register(TIMER_ACTOR_NAME).unwrap();
        actor_ref.link(&timer).await;
        let wire_factory = WireFactoryActor::spawn(WireFactoryActor::new(
            args.domain_id,
            args.configuration.clone(),
        ));
        wire_factory.wait_for_startup().await;
        wire_factory.register(WIRE_FACTORY_ACTOR_NAME).unwrap();
        actor_ref.link(&wire_factory).await;
        let discovery = DiscoveryActor::spawn(DiscoveryActor::new());
        discovery.wait_for_startup().await;
        discovery.register(DISCOVERY_ACTOR_NAME).unwrap();
        actor_ref.link(&discovery).await;
        let entity_identifier = EntityIdentifierActor::spawn(());
        entity_identifier.wait_for_startup().await;
        entity_identifier
            .register(ENTITY_IDENTIFIER_ACTOR_NAME)
            .unwrap();
        actor_ref.link(&entity_identifier).await;
        let domain_participant_actor = Self {
            domain_id: args.domain_id,
            guid: args.guid,
            // listener: todo!(),
            infos: args.infos,
            config: args.configuration,
            timer,
            wire_factory,
            discovery,
            entity_identifier,
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
        todo!()
    }

    async fn on_link_died(
        &mut self,
        actor_ref: kameo::prelude::WeakActorRef<Self>,
        id: kameo::prelude::ActorId,
        reason: kameo::prelude::ActorStopReason,
    ) -> Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error> {
        // relaunch dead Actor if they didn't die on purpose
        todo!()
    }
}

impl DomainParticipantActor {
    // pub async fn create_subscriber(&mut self, qos: &QosPolicy) -> Result<Subscriber, DdsError> {
    //     event!(Level::TRACE, "subscriber creation...");

    //     let mut participant_inner_guard = self.lock_inner().await;

    //     let sub_key = participant_inner_guard
    //         .subscriber_entity_identifier
    //         .get_new_key();

    //     let sub_id = EntityId::reader_group_builtin(sub_key.0);

    //     let sub_guid = Guid::new(self.guid.get_guid_prefix(), sub_id);

    //     let subscriber = {
    //         let (default_unicast_locator_list, default_multicast_locator_list) = {
    //             let infos_guard = participant_inner_guard.infos.lock().await;

    //             let default_unicast_locator_list = infos_guard.get_default_unicast_locator_list();
    //             let default_multicast_locator_list =
    //                 infos_guard.get_default_multicast_locator_list();

    //             (default_unicast_locator_list, default_multicast_locator_list)
    //         };

    //         let participant = self.get_weak();
    //         let disc_cmd_sender = participant_inner_guard.disc_cmd_sender.clone();
    //         let reader_entity_identifier = participant_inner_guard.reader_entity_identifier.clone();
    //         let config = participant_inner_guard.config.clone();

    //         let subscriber = Subscriber::new(
    //             sub_guid,
    //             default_unicast_locator_list,
    //             default_multicast_locator_list,
    //             participant,
    //             disc_cmd_sender,
    //             reader_entity_identifier,
    //             *qos,
    //             self.wire_factory.clone(),
    //             config,
    //             self.span.clone(),
    //         );

    //         let old = participant_inner_guard
    //             .subscribers
    //             .insert(subscriber.get_entity_id(), subscriber.inner.clone());

    //         assert!(old.is_none());

    //         subscriber
    //     };
    //     Ok(subscriber)
    // }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use kameo::{
        Actor,
        actor::{ActorRef, Spawn},
        error::Infallible,
        prelude::Message,
    };
    use rstest::rstest;
    use tokio::time::sleep;

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
