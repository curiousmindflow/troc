use std::{collections::HashMap, marker::PhantomData};

use crate::{
    discovery::DiscoveryActor,
    infrastructure::QosPolicy,
    publication::DataWriterListener,
    time::{TimerActor, TimerActorMessage},
    wires::{WireActor, WireActorMessage},
};
use bytes::BytesMut;
use kameo::{Actor, actor::ActorRef, prelude::Message};
use serde::Serialize;

use tracing::{Level, event};
use troc_core::{DdsError, Effect, ReaderProxy, Writer};
use troc_core::{Effects, Keyed};
use troc_core::{
    serialize_data,
    types::{ChangeKind, Guid, InlineQos, InstanceHandle, Locator, SequenceNumber, SerializedData},
};

#[derive(Debug)]
pub struct DataWriter<T> {
    guid: Guid,
    qos: InlineQos,
    data_writer_actor: ActorRef<DataWriterActor>,
    phantom: PhantomData<T>,
}

impl<T> DataWriter<T> {
    pub(crate) async fn new(
        guid: Guid,
        qos: QosPolicy,
        data_writer_actor: ActorRef<DataWriterActor>,
    ) -> Self {
        Self {
            guid,
            qos: qos.into(),
            data_writer_actor,
            phantom: PhantomData,
        }
    }

    pub fn get_guid(&self) -> Guid {
        self.guid
    }

    pub async fn get_listener(&self) -> Option<DataWriterListener> {
        unimplemented!()
    }

    pub async fn write(&mut self, data: T) -> Result<SequenceNumber, DdsError>
    where
        T: Serialize + Keyed,
    {
        let key = data.key().unwrap();
        let data = serialize_data(&data).unwrap();
        self.write_raw(data, InstanceHandle(key)).await
    }

    pub async fn write_raw(
        &mut self,
        data: SerializedData,
        key: InstanceHandle,
    ) -> Result<SequenceNumber, DdsError> {
        self.data_writer_actor
            .tell(DataWriterActorMessage::Write {
                data,
                instance: key,
            })
            .await
            .unwrap();
        todo!()
    }

    // TODO: should not be used
    // instead leverages the QoS
    pub async fn remove_sample(&self, sequence: SequenceNumber) {
        unimplemented!()
    }

    // pub fn incomming_task(&self, mut wire: Wire) {
    //     let writer = self.writer.clone();
    //     let cancellation_token = self.cancellation_token.child_token();
    //     tokio::spawn(async move {
    //         loop {
    //             select! {
    //                     _ = cancellation_token.cancelled() => {
    //                         break
    //                     }
    //                     Ok(buffer) = wire.recv() => {
    //                         let Ok(deserialization_result) = Handle::current()
    //                             .spawn_blocking(move || Message::deserialize_from(&buffer))
    //                             .await
    //                         else {
    //                             event!(Level::ERROR, "Spawn blocking failed");
    //                             break;
    //                         };
    //                         match deserialization_result {
    //                             Ok(message) => {
    //                                 let message = IncommingMessage::new(message);
    //                                 if let Err(e) = writer.lock().unwrap().ingest(message) {
    //                                     event!(Level::ERROR, "{e}");
    //                                 }
    //                             }
    //                             Err(e) => {
    //                                 event!(Level::ERROR, "{e}");
    //                             }
    //                         }
    //                     }
    //                     else => {
    //                         event!(Level::ERROR, "Incomming task terminate");
    //                         break;
    //                     }
    //             }
    //         }
    //     });
    // }
}

#[derive(Debug)]
pub enum DataWriterActorMessage {
    Write {
        data: SerializedData,
        instance: InstanceHandle,
    },
    Message {
        message: troc_core::messages::Message,
    },
    AddProxy {
        proxy: ReaderProxy,
        wires: HashMap<Locator, ActorRef<WireActor>>,
    },
    RemoveProxy {
        guid: Guid,
        locators: Vec<Locator>,
    },
    Tick,
}

impl Message<DataWriterActorMessage> for DataWriterActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: DataWriterActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            DataWriterActorMessage::Write { data, instance } => {
                let change = self
                    .writer
                    .new_change(ChangeKind::Alive, Some(data), None, instance);
                self.writer.add_change(&mut self.effects, change).unwrap();
            }
            DataWriterActorMessage::Message { message } => {
                self.writer.ingest(&mut self.effects, message).unwrap()
            }
            DataWriterActorMessage::AddProxy { proxy, wires } => {
                self.output_wires.extend(wires);
                self.writer.add_proxy(proxy)
            }
            DataWriterActorMessage::RemoveProxy { guid, locators } => {
                for locator in locators {
                    self.output_wires.remove(&locator);
                }
                self.writer.remove_proxy(guid)
            }
            DataWriterActorMessage::Tick => self.writer.tick(&mut self.effects),
        }

        while let Some(effect) = self.effects.pop() {
            match effect {
                Effect::Message {
                    timestamp_millis,
                    message,
                    locators,
                } => {
                    let (nb_bytes, message) = tokio::task::spawn_blocking(move || {
                        // TODO: use a Memory pool to avoid creating a buffer each time serialization ocurrs
                        let mut emission_buffer = BytesMut::with_capacity(65 * 1024);
                        let nb_bytes = message.serialize_to(&mut emission_buffer).unwrap();
                        (nb_bytes, emission_buffer)
                    })
                    .await
                    .unwrap();
                    for locator in locators.iter() {
                        if let Some(actor) = self.output_wires.get(locator) {
                            actor
                                .tell(WireActorMessage::Send(message.clone()))
                                .await
                                .unwrap();
                        }
                    }
                }
                Effect::ScheduleTick { delay } => {
                    self.timer
                        .tell(TimerActorMessage::ScheduleWriterTick {
                            delay,
                            target: ctx.actor_ref().clone(),
                        })
                        .await
                        .unwrap();
                    todo!()
                }
                Effect::Qos => todo!(),
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Debug)]
pub struct DataWriterActorCreateObject {
    pub writer: Writer,
    pub input_wires: Vec<ActorRef<WireActor>>,
}

#[derive(Debug)]
pub struct DataWriterActor {
    writer: Writer,
    effects: Effects,
    discovery: ActorRef<DiscoveryActor>,
    timer: ActorRef<TimerActor>,
    input_wires: Vec<ActorRef<WireActor>>,
    output_wires: HashMap<Locator, ActorRef<WireActor>>,
}

impl Actor for DataWriterActor {
    type Args = DataWriterActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let DataWriterActorCreateObject {
            writer,
            input_wires,
        } = args;

        let discovery = ActorRef::<DiscoveryActor>::lookup("discovery")
            .unwrap()
            .unwrap();

        let timer = ActorRef::<TimerActor>::lookup("timer").unwrap().unwrap();

        let datawriter_actor = Self {
            writer,
            effects: Effects::default(),
            discovery,
            timer,
            input_wires,
            output_wires: Default::default(),
        };

        Ok(datawriter_actor)
    }

    async fn on_stop(
        &mut self,
        actor_ref: kameo::prelude::WeakActorRef<Self>,
        reason: kameo::prelude::ActorStopReason,
    ) -> Result<(), Self::Error> {
        // TODO: must tell discovery
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::*;
    use tokio::sync::mpsc::channel;
    use troc_core::types::{Guid, Locator};

    use crate::{
        domain::Configuration, infrastructure::QosPolicy, publication::datawriter::DataWriter,
    };

    // #[fixture]
    // async fn new() -> DataWriter<()> {
    //     let guid = Guid::default();
    //     let qos = QosPolicy::default();

    //     let (endpoint_lifecycle_sender, _endpoint_lifecycle_receiver) = channel(1024);
    //     let datawriter = DataWriter::new(guid, qos, endpoint_lifecycle_sender).await;

    //     let wire_factory = WireFactory::new(0, Configuration::default());
    //     let locator = Locator::from_str("127.0.0.1:65000:UDPV4").unwrap();
    //     let wire = wire_factory
    //         .build_listener_wire_from_locator(&locator)
    //         .unwrap();
    //     datawriter.incomming_task(wire);
    //     datawriter
    // }
}
