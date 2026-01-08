use std::time::Duration;

use kameo::{Actor, actor::ActorRef, error::Infallible, prelude::Message};
use tokio::time::sleep;

use crate::publication::{DataWriterActor, DataWriterActorMessage};

#[derive()]
pub enum TimerActorMessage {
    ScheduleWriterTick {
        delay: i64,
        target: ActorRef<DataWriterActor>,
    },
}

impl Message<TimerActorMessage> for TimerActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: TimerActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            TimerActorMessage::ScheduleWriterTick { delay, target } => {
                tokio::spawn(async move {
                    sleep(Duration::from_millis(delay as u64));
                    target.tell(DataWriterActorMessage::Tick).await.unwrap()
                });
            }
        }
    }
}

#[derive(Debug)]
pub struct TimerActor {}

impl Actor for TimerActor {
    type Args = Self;

    type Error = Infallible;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TimerActor {
    pub fn new() -> Self {
        todo!()
    }
}
