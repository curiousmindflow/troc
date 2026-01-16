use std::sync::Arc;

use kameo::{
    Actor,
    actor::{ActorRef, Spawn},
    prelude::Message,
};
use serde::Deserialize;
use tokio::sync::Notify;
use troc_core::{
    DdsError, EntityId, EntityKey, Guid, GuidPrefix, Keyed, LocatorList, ReaderBuilder,
    ReaderProxy, TopicKind,
};

use crate::{
    discovery::{DiscoveryActor, DiscoveryActorMessage},
    domain::{
        Configuration, DISCOVERY_ACTOR_NAME, ENTITY_IDENTIFIER_ACTOR_NAME, EntityIdentifierActor,
        EntityIdentifierActorAskMessage, WIRE_FACTORY_ACTOR_NAME,
    },
    infrastructure::QosPolicy,
    subscription::{
        DataReader, DataReaderActor, DataReaderActorCreateObject, DataReaderActorMessage,
    },
    topic::Topic,
    wires::{
        ReceiverWireFactoryActorMessage, ReceiverWireFactoryActorMessageDestKind, WireFactoryActor,
    },
};

#[derive(Clone)]
pub struct Subscriber {
    guid: Guid,
    qos: QosPolicy,
    subscriber_actor: ActorRef<SubscriberActor>,
}

impl Subscriber {
    pub(crate) fn new(
        guid: Guid,
        default_unicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        qos: QosPolicy,
        config: Configuration,
        subscriber_actor: ActorRef<SubscriberActor>,
    ) -> Self {
        Self {
            guid,
            qos,
            subscriber_actor,
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
        let entity_identifier_actore =
            ActorRef::<EntityIdentifierActor>::lookup(ENTITY_IDENTIFIER_ACTOR_NAME)
                .unwrap()
                .unwrap();
        let wire_factory = ActorRef::<WireFactoryActor>::lookup(WIRE_FACTORY_ACTOR_NAME)
            .unwrap()
            .unwrap();

        let writer_key: EntityKey = entity_identifier_actore
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

        let reader = ReaderBuilder::new(reader_guid, (*qos).into())
            .reliability(reliable)
            .build();
        let reader_proxy = reader.extract_proxy();

        let data_availability_notifier = Arc::new(Notify::new());
        let reader_actor = DataReaderActor::spawn(DataReaderActorCreateObject {
            reader,
            data_availability_notifier: data_availability_notifier.clone(),
        });

        let (input_wires, locators) = wire_factory
            .ask(ReceiverWireFactoryActorMessage::<DataReaderActor>::new(
                ReceiverWireFactoryActorMessageDestKind::Applicative,
                reader_actor.clone(),
            ))
            .await
            .unwrap();

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
                writer: reader_actor,
            })
            .await
            .unwrap();

        Ok(datareader)
    }
}

#[derive(Debug)]
pub struct SubscriberActorMessage {
    proxy: ReaderProxy,
    writer: ActorRef<DataReaderActor>,
}

impl Message<SubscriberActorMessage> for SubscriberActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: SubscriberActorMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.readers.push(msg.writer);
        self.discovery
            .ask(DiscoveryActorMessage::ReaderCreated {
                reader_proxy: msg.proxy,
            })
            .await
            .unwrap();
    }
}

#[derive(Debug)]
pub struct SubscriberActorCreateObject {}

#[derive(Debug)]
pub struct SubscriberActor {
    readers: Vec<ActorRef<DataReaderActor>>,
    discovery: ActorRef<DiscoveryActor>,
}

impl Actor for SubscriberActor {
    type Args = SubscriberActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        _args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let discovery = ActorRef::<DiscoveryActor>::lookup(DISCOVERY_ACTOR_NAME)
            .unwrap()
            .unwrap();

        let subscriber_actor = Self {
            readers: Default::default(),
            discovery,
        };

        Ok(subscriber_actor)
    }
}
