use kameo::Actor;
use kameo::actor::ActorRef;
use kameo::actor::Spawn;
use kameo::prelude::Message;
use serde::Serialize;
use troc_core::DdsError;
use troc_core::EntityKey;
use troc_core::Keyed;
use troc_core::WriterBuilder;
use troc_core::WriterProxy;
use troc_core::{EntityId, Guid, GuidPrefix, LocatorList, TopicKind};

use crate::discovery::DiscoveryActor;
use crate::discovery::DiscoveryActorMessage;
use crate::domain::EntityIdentifierActorAskMessage;
use crate::publication::DataWriterActor;
use crate::publication::DataWriterActorMessage;
use crate::publication::datawriter::DataWriterActorCreateObject;
use crate::time::TimerActor;
use crate::wires::ReceiverWireFactoryActorMessage;
use crate::wires::WireFactoryActor;
use crate::{
    domain::{Configuration, EntityIdentifierActor},
    infrastructure::QosPolicy,
    publication::DataWriter,
    topic::Topic,
};

#[derive(Clone)]
pub struct Publisher {
    guid: Guid,
    qos: QosPolicy,
    publisher_actor: ActorRef<PublisherActor>,
    wire_factory: ActorRef<WireFactoryActor>,
    discovery: ActorRef<DiscoveryActor>,
    entity_identifier: ActorRef<EntityIdentifierActor>,
    timer: ActorRef<TimerActor>,
}

impl Publisher {
    pub(crate) fn new(
        guid: Guid,
        default_unicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        qos: QosPolicy,
        config: Configuration,
        publisher_actor: ActorRef<PublisherActor>,
        wire_factory: ActorRef<WireFactoryActor>,
        discovery: ActorRef<DiscoveryActor>,
        entity_identifier: ActorRef<EntityIdentifierActor>,
        timer: ActorRef<TimerActor>,
    ) -> Self {
        Self {
            guid,
            qos,
            publisher_actor,
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

    pub async fn create_datawriter<T>(
        &mut self,
        topic: &Topic<T>,
        qos: &QosPolicy,
    ) -> Result<DataWriter<T>, DdsError>
    where
        T: Serialize + Keyed + 'static,
    {
        let writer_key: EntityKey = self
            .entity_identifier
            .ask(EntityIdentifierActorAskMessage::AskWriterId)
            .await
            .unwrap()
            .into();

        let writer_id = if matches!(topic.topic_kind, TopicKind::WithKey) {
            EntityId::writer_with_key(writer_key.0)
        } else {
            EntityId::writer_no_key(writer_key.0)
        };
        let writer_guid = Guid::new(self.guid.get_guid_prefix(), writer_id);

        let reliable = qos.reliability().into();

        let writer = WriterBuilder::new(writer_guid, (*qos).into())
            .reliability(reliable)
            .build();
        let writer_proxy = writer.extract_proxy();
        let writer_actor = DataWriterActor::spawn(DataWriterActorCreateObject {
            writer,
            discovery: self.discovery.clone(),
            timer: self.timer.clone(),
        });

        let (input_wires, locators) = self
            .wire_factory
            .ask(ReceiverWireFactoryActorMessage::Applicative)
            .await
            .unwrap();

        let datawriter = DataWriter::new(writer_guid, *qos, writer_actor.clone()).await;

        writer_actor
            .ask(DataWriterActorMessage::AddInputWire {
                wires: input_wires,
                locators,
            })
            .await
            .unwrap();

        self.publisher_actor
            .ask(PublisherActorMessage {
                proxy: writer_proxy,
                writer: writer_actor,
            })
            .await
            .unwrap();

        Ok(datawriter)
    }
}

#[derive(Debug)]
pub struct PublisherActorMessage {
    proxy: WriterProxy,
    writer: ActorRef<DataWriterActor>,
}

impl Message<PublisherActorMessage> for PublisherActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: PublisherActorMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.writers.push(msg.writer);
        self.discovery
            .ask(DiscoveryActorMessage::WriterCreated {
                writer_proxy: msg.proxy,
            })
            .await
            .unwrap();
    }
}

#[derive(Debug)]
pub struct PublisherActorCreateObject {
    pub discovery: ActorRef<DiscoveryActor>,
}

#[derive(Debug)]
pub struct PublisherActor {
    writers: Vec<ActorRef<DataWriterActor>>,
    discovery: ActorRef<DiscoveryActor>,
}

impl Actor for PublisherActor {
    type Args = PublisherActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let publisher_actor = Self {
            writers: Default::default(),
            discovery: args.discovery,
        };

        Ok(publisher_actor)
    }
}
