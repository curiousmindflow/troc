use std::time::Duration;

use kameo::{Actor, actor::ActorRef, error::Infallible, prelude::Message};
use tokio::time::sleep;
use tracing::{Level, event, instrument};
use troc_core::TickId;

use crate::{
    discovery::{DiscoveryActor, DiscoveryActorMessage},
    publication::{DataWriterActor, DataWriterActorMessage},
    subscription::{DataReaderActor, DataReaderActorMessage},
};

#[derive()]
pub enum TimerActorScheduleTickMessage {
    Writer {
        delay: i64,
        target: ActorRef<DataWriterActor>,
    },
    Reader {
        delay: i64,
        target: ActorRef<DataReaderActor>,
    },
    Discovery {
        delay: i64,
        target: ActorRef<DiscoveryActor>,
        id: TickId,
    },
}

impl Message<TimerActorScheduleTickMessage> for TimerActor {
    type Reply = ();

    #[instrument(name = "timer", skip_all, fields())]
    async fn handle(
        &mut self,
        msg: TimerActorScheduleTickMessage,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            TimerActorScheduleTickMessage::Writer { delay, target } => {
                tokio::spawn(async move {
                    event!(
                        Level::WARN,
                        %delay,
                        ?target,
                        "TimerActorScheduleTickMessage::Writer received"
                    );
                    sleep(Duration::from_millis(delay as u64)).await;
                    event!(
                        Level::WARN,
                        %delay,
                        ?target,
                        "TimerActorScheduleTickMessage::Writer delay reached"
                    );
                    target.tell(DataWriterActorMessage::Tick).await.unwrap()
                });
            }
            TimerActorScheduleTickMessage::Reader { delay, target } => {
                tokio::spawn(async move {
                    event!(
                        Level::WARN,
                        %delay,
                        ?target,
                        "TimerActorScheduleTickMessage::Reader received"
                    );
                    sleep(Duration::from_millis(delay as u64)).await;
                    event!(
                        Level::WARN,
                        %delay,
                        ?target,
                        "TimerActorScheduleTickMessage::Reader delay reached"
                    );
                    target.tell(DataReaderActorMessage::Tick).await.unwrap()
                });
            }
            TimerActorScheduleTickMessage::Discovery { delay, target, id } => {
                tokio::spawn(async move {
                    event!(
                        Level::WARN,
                        %delay,
                        ?target,
                        ?id,
                        "TimerActorScheduleTickMessage::Discovery received"
                    );
                    sleep(Duration::from_millis(delay as u64)).await;
                    event!(
                        Level::WARN,
                        %delay,
                        ?target,
                        "TimerActorScheduleTickMessage::Discovery delay reached"
                    );
                    target.tell(DiscoveryActorMessage::Tick(id)).await.unwrap()
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
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(args)
    }
}

impl TimerActor {
    pub fn new() -> Self {
        Self {}
    }
}
