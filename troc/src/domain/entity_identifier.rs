use std::collections::BTreeSet;

use kameo::{Actor, Reply, error::Infallible, prelude::Message};
use troc_core::EntityKey;

#[derive(Debug)]
pub enum EntityIdentifierActorAskMessage {
    AskWriterId,
    AskReaderId,
    AskPublisherId,
    AskSubscriberId,
}

#[derive(Debug)]
pub enum EntityIdentifierActorFreeMessage {
    FreeWriterId(EntityKey),
    FreeReaderId(EntityKey),
    FreePublisherrId(EntityKey),
    FreeSubscriberId(EntityKey),
}

#[derive(Debug, Reply)]
pub struct AskedId(EntityKey);

impl From<AskedId> for EntityKey {
    fn from(value: AskedId) -> Self {
        value.0
    }
}

impl Message<EntityIdentifierActorAskMessage> for EntityIdentifierActor {
    type Reply = AskedId;

    async fn handle(
        &mut self,
        msg: EntityIdentifierActorAskMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            EntityIdentifierActorAskMessage::AskWriterId => self.get_new_writer_key(),
            EntityIdentifierActorAskMessage::AskReaderId => self.get_new_reader_key(),
            EntityIdentifierActorAskMessage::AskPublisherId => self.get_new_writer_key(),
            EntityIdentifierActorAskMessage::AskSubscriberId => self.get_new_reader_key(),
        }
    }
}

impl Message<EntityIdentifierActorFreeMessage> for EntityIdentifierActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: EntityIdentifierActorFreeMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            EntityIdentifierActorFreeMessage::FreeWriterId(key) => self.remove_writer_key(key),
            EntityIdentifierActorFreeMessage::FreeReaderId(key) => self.remove_reader_key(key),
            EntityIdentifierActorFreeMessage::FreePublisherrId(key) => self.remove_reader_key(key),
            EntityIdentifierActorFreeMessage::FreeSubscriberId(key) => self.remove_reader_key(key),
        }
    }
}

#[derive(Debug)]
pub struct EntityIdentifierActor {
    writer_history: BTreeSet<EntityKey>,
    reader_history: BTreeSet<EntityKey>,
    publisher_history: BTreeSet<EntityKey>,
    subscriber_history: BTreeSet<EntityKey>,
}

impl EntityIdentifierActor {
    fn get_new_writer_key(&mut self) -> AskedId {
        Self::get_new_key(&mut self.writer_history)
    }

    fn remove_writer_key(&mut self, key: EntityKey) {
        Self::remove_key(&mut self.writer_history, key);
    }

    fn get_new_reader_key(&mut self) -> AskedId {
        Self::get_new_key(&mut self.reader_history)
    }

    fn remove_reader_key(&mut self, key: EntityKey) {
        Self::remove_key(&mut self.reader_history, key);
    }

    fn get_new_publisher_key(&mut self) -> AskedId {
        Self::get_new_key(&mut self.publisher_history)
    }

    fn remove_publisher_key(&mut self, key: EntityKey) {
        Self::remove_key(&mut self.publisher_history, key);
    }

    fn get_new_subscriber_key(&mut self) -> AskedId {
        Self::get_new_key(&mut self.subscriber_history)
    }

    fn remove_subscriber_key(&mut self, key: EntityKey) {
        Self::remove_key(&mut self.subscriber_history, key);
    }

    fn get_new_key(history: &mut BTreeSet<EntityKey>) -> AskedId {
        for possible_key in (EntityKey::MIN..=EntityKey::MAX).map(EntityKey::from_u32) {
            if !history.contains(&possible_key) {
                history.insert(possible_key);
                return AskedId(possible_key);
            }
        }
        unreachable!()
    }

    fn remove_key(history: &mut BTreeSet<EntityKey>, key: EntityKey) {
        history.remove(&key);
    }
}

impl Actor for EntityIdentifierActor {
    type Args = ();

    type Error = Infallible;

    async fn on_start(
        _args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let entitiy_identifier = Self {
            writer_history: Default::default(),
            reader_history: Default::default(),
            publisher_history: Default::default(),
            subscriber_history: Default::default(),
        };
        Ok(entitiy_identifier)
    }
}
