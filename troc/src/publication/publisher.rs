use kameo::Actor;
use kameo::actor::ActorRef;
use kameo::actor::Spawn;
use kameo::prelude::Message;
use serde::Serialize;
use troc_core::DdsError;
use troc_core::Keyed;
use troc_core::WriterBuilder;
use troc_core::WriterConfiguration;
use troc_core::WriterProxy;
use troc_core::types::EntityKey;
use troc_core::types::{EntityId, Guid, GuidPrefix, LocatorList, TopicKind};

use crate::domain::ENTITY_IDENTIFIER_ACTOR_NAME;
use crate::domain::EntityIdentifierActorAskMessage;
use crate::domain::WIRE_FACTORY_ACTOR_NAME;
use crate::publication::DataWriterActor;
use crate::publication::datawriter::DataWriterActorCreateObject;
use crate::wires::WireFactoryActor;
use crate::wires::WireFactoryActorMessage;
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
}

impl Publisher {
    pub(crate) fn new(
        guid: Guid,
        default_unicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        qos: QosPolicy,
        config: Configuration,
        publisher_actor: ActorRef<PublisherActor>,
    ) -> Self {
        Self {
            guid,
            qos,
            publisher_actor,
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
        let entity_identifier_actore =
            ActorRef::<EntityIdentifierActor>::lookup(ENTITY_IDENTIFIER_ACTOR_NAME)
                .unwrap()
                .unwrap();
        let writer_key: EntityKey = entity_identifier_actore
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

        let config = WriterConfiguration::default();
        let writer = WriterBuilder::new(writer_guid, (*qos).into(), config)
            .reliable(reliable)
            .build();
        let writer_proxy = writer.extract_proxy();

        let wire_factory = ActorRef::<WireFactoryActor>::lookup(WIRE_FACTORY_ACTOR_NAME)
            .unwrap()
            .unwrap();

        let mut input_wires = Vec::default();

        let localhost_wires = wire_factory
            .ask(WireFactoryActorMessage::CreateLocalhostInputWires)
            .await
            .unwrap();
        input_wires.extend(localhost_wires);
        let machine_wires = wire_factory
            .ask(WireFactoryActorMessage::CreateInputWires)
            .await
            .unwrap();
        input_wires.extend(machine_wires);

        let writer_actor = DataWriterActor::spawn(DataWriterActorCreateObject {
            writer,
            input_wires,
        });

        let datawriter = DataWriter::new(writer_guid, *qos, writer_actor.clone()).await;

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
        // TODO: send info to Discovery
    }
}

#[derive(Debug)]
pub struct PublisherActorCreateObject {}

#[derive(Debug)]
pub struct PublisherActor {
    writers: Vec<ActorRef<DataWriterActor>>,
}

impl Actor for PublisherActor {
    type Args = PublisherActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        _args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let publisher_actor = Self {
            writers: Default::default(),
        };

        Ok(publisher_actor)
    }
}
