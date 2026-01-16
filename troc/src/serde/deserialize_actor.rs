use std::marker::PhantomData;

use bytes::BytesMut;
use kameo::{Actor, actor::ActorRef, prelude::Message};
use serde::de::DeserializeOwned;
use troc_core::{
    DdsError,
    cdr::{self},
};

pub type DeserializeActorReply<T> = Result<T, DdsError>;

#[derive(Debug)]
pub struct DeserializeActorMessage<T, A>
where
    T: Send + 'static,
    A: Actor,
{
    buffer: BytesMut,
    recipient: ActorRef<A>,
    _phantom: PhantomData<T>,
}

impl<T, A> Message<DeserializeActorMessage<T, A>> for DeserializeActor
where
    T: DeserializeOwned + Send + 'static,
    A: Actor,
{
    type Reply = DeserializeActorReply<T>;

    async fn handle(
        &mut self,
        msg: DeserializeActorMessage<T, A>,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let res = cdr::deserialize(&msg.buffer).unwrap();
        Ok(res)
    }
}

#[derive(Debug)]
pub struct DeserializeActor {}

impl Actor for DeserializeActor {
    type Args = Self;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}
