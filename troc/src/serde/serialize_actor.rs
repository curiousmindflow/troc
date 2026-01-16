use bytes::BytesMut;
use kameo::Actor;
use troc_core::DdsError;

#[derive(Debug)]
pub struct SerializeActorMessage {
    buffer: BytesMut,
}

#[derive(Debug)]
pub struct SerializeActor {
    //
}

impl Actor for SerializeActor {
    type Args = Self;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}
