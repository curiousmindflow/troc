use std::sync::Arc;

use kameo::{
    Actor,
    actor::{ActorRef, Spawn},
    prelude::Message,
};
use serde::Deserialize;
use tokio::sync::Notify;
use troc_core::{
    DdsError, DiscoveredReaderData, EntityId, EntityKey, Guid, GuidPrefix, InlineQos, Keyed,
    LocatorList, ReaderBuilder, ReaderProxy, TopicKind,
};

use crate::{
    discovery::{DiscoveryActor, DiscoveryActorMessage},
    domain::{Configuration, EntityIdentifierActor, EntityIdentifierActorAskMessage},
    infrastructure::QosPolicy,
    subscription::{
        DataReader, DataReaderActor, DataReaderActorCreateObject, DataReaderActorMessage,
    },
    time::TimerActor,
    topic::Topic,
    wires::{ReceiverWireFactoryActorMessage, WireFactoryActor},
};

#[derive(Clone)]
pub struct Subscriber {
    guid: Guid,
    qos: QosPolicy,
    subscriber_actor: ActorRef<SubscriberActor>,
    wire_factory: ActorRef<WireFactoryActor>,
    discovery: ActorRef<DiscoveryActor>,
    entity_identifier: ActorRef<EntityIdentifierActor>,
    timer: ActorRef<TimerActor>,
}

impl Subscriber {
    pub(crate) fn new(
        guid: Guid,
        default_unicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        qos: QosPolicy,
        config: Configuration,
        subscriber_actor: ActorRef<SubscriberActor>,
        wire_factory: ActorRef<WireFactoryActor>,
        discovery: ActorRef<DiscoveryActor>,
        entity_identifier: ActorRef<EntityIdentifierActor>,
        timer: ActorRef<TimerActor>,
    ) -> Self {
        Self {
            guid,
            qos,
            subscriber_actor,
            wire_factory,
            discovery,
            entity_identifier,
            timer,
        }
    }

    pub fn get_guid_prefix(&self) -> GuidPrefix {
        self.guid.get_guid_prefix()
    }

    pub fn get_entity_id(&self) -> EntityId {
        self.guid.get_entity_id()
    }

    pub async fn create_datareader<T>(
        &mut self,
        topic: &Topic<T>,
        qos: &QosPolicy,
    ) -> Result<DataReader<T>, DdsError>
    where
        for<'a> T: Deserialize<'a> + Keyed + 'static,
    {
        let writer_key: EntityKey = self
            .entity_identifier
            .ask(EntityIdentifierActorAskMessage::AskReaderId)
            .await
            .unwrap()
            .into();

        let writer_id = if matches!(topic.topic_kind, TopicKind::WithKey) {
            EntityId::writer_with_key(writer_key.0)
        } else {
            EntityId::writer_no_key(writer_key.0)
        };
        let reader_guid = Guid::new(self.guid.get_guid_prefix(), writer_id);

        let reliable = qos.reliability().into();
        let mut inline_qos: InlineQos = (*qos).into();
        inline_qos.topic_name = topic.topic_name.clone();
        inline_qos.type_name = topic.type_name.clone();

        let (input_wires, locators) = self
            .wire_factory
            .ask(ReceiverWireFactoryActorMessage::Applicative)
            .await
            .unwrap();

        let reader = ReaderBuilder::new(reader_guid, inline_qos.clone())
            .reliability(reliable)
            .with_unicast_locators(locators.clone())
            .build();
        let reader_proxy = reader.extract_proxy();

        let data_availability_notifier = Arc::new(Notify::new());
        let reader_actor = DataReaderActor::spawn(DataReaderActorCreateObject {
            reader,
            qos: inline_qos.clone(),
            data_availability_notifier: data_availability_notifier.clone(),
            discovery: self.discovery.clone(),
            timer: self.timer.clone(),
        });

        let datareader = DataReader::new(
            reader_guid,
            *qos,
            reader_actor.clone(),
            data_availability_notifier,
        )
        .await;

        reader_actor
            .ask(DataReaderActorMessage::AddInputWire {
                wires: input_wires,
                locators,
            })
            .await
            .unwrap();

        self.subscriber_actor
            .ask(SubscriberActorMessage {
                proxy: reader_proxy,
                qos: inline_qos,
                readers: reader_actor,
            })
            .await
            .unwrap();

        Ok(datareader)
    }
}

#[derive(Debug)]
pub struct SubscriberActorMessage {
    proxy: ReaderProxy,
    qos: InlineQos,
    readers: ActorRef<DataReaderActor>,
}

impl Message<SubscriberActorMessage> for SubscriberActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: SubscriberActorMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.readers.push(msg.readers.clone());
        let disc_reader_data = DiscoveredReaderData {
            proxy: msg.proxy,
            params: msg.qos,
        };
        self.discovery
            .ask(DiscoveryActorMessage::ReaderCreated {
                reader_discovery_data: disc_reader_data,
                actor: msg.readers,
            })
            .await
            .unwrap();
    }
}

#[derive(Debug)]
pub struct SubscriberActorCreateObject {
    pub discovery: ActorRef<DiscoveryActor>,
}

#[derive(Debug)]
pub struct SubscriberActor {
    readers: Vec<ActorRef<DataReaderActor>>,
    discovery: ActorRef<DiscoveryActor>,
}

impl Actor for SubscriberActor {
    type Args = SubscriberActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let subscriber_actor = Self {
            readers: Default::default(),
            discovery: args.discovery,
        };

        Ok(subscriber_actor)
    }
}
