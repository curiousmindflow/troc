use std::{collections::HashMap, marker::PhantomData};

use crate::{
    DataWriterEvent,
    infrastructure::QosPolicy,
    publication::DataWriterListener,
    time::{TimerActor, TimerActorScheduleTickMessage},
    wires::{
        ReceiverWireActor, ReceiverWireActorMessage, Sendable, SenderWireActor,
        SenderWireActorMessage,
    },
};
use bytes::BytesMut;
use chrono::Utc;
use kameo::{Actor, actor::ActorRef, prelude::Message};
use serde::Serialize;

use tokio::sync::broadcast::{Receiver, Sender, channel};
use tracing::{Level, event, instrument};
use troc_core::{
    ChangeKind, Guid, InlineQos, InstanceHandle, Locator, LocatorList, SequenceNumber,
    SerializedData, cdr,
};
use troc_core::{
    DdsError, Effect, ReaderProxy, Writer,
    cdr::{CdrLe, Infinite},
};
use troc_core::{Effects, Keyed};

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

    pub async fn get_listener(&self) -> Result<DataWriterListener, DdsError> {
        let receiver = self
            .data_writer_actor
            .ask(DataWriterListenerCreate {})
            .await
            .unwrap();
        Ok(DataWriterListener { receiver })
    }

    pub async fn write(&mut self, data: T) -> Result<(), DdsError>
    where
        T: Serialize + Keyed,
    {
        let key = data.key().unwrap();
        let data = cdr::serialize::<_, _, CdrLe>(&data, Infinite).unwrap();
        let data = SerializedData::from_vec(data);
        self.write_raw(data, InstanceHandle(key)).await.unwrap();
        Ok(())
    }

    pub async fn write_raw(
        &mut self,
        data: SerializedData,
        key: InstanceHandle,
    ) -> Result<(), DdsError> {
        self.data_writer_actor
            .tell(DataWriterActorMessage::Write {
                data,
                instance: key,
            })
            .await
            .unwrap();
        Ok(())
    }

    // TODO: should not be used
    // instead leverages the QoS
    pub async fn remove_sample(&self, sequence: SequenceNumber) {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct DataWriterListenerCreate;

impl Message<DataWriterListenerCreate> for DataWriterActor {
    type Reply = Receiver<DataWriterEvent>;

    async fn handle(
        &mut self,
        _msg: DataWriterListenerCreate,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self._event_receiver.take().unwrap()
    }
}

#[derive(Debug)]
pub enum DataWriterActorMessage {
    Write {
        data: SerializedData,
        instance: InstanceHandle,
    },
    IncomingMessage {
        message: BytesMut,
    },
    AddProxy {
        proxy: ReaderProxy,
        wires: HashMap<Locator, ActorRef<SenderWireActor>>,
    },
    RemoveProxy {
        guid: Guid,
        locators: Vec<Locator>,
    },
    Tick,
    AddInputWire {
        wires: Vec<ActorRef<ReceiverWireActor>>,
        locators: LocatorList,
    },
}

impl Message<DataWriterActorMessage> for DataWriterActor {
    type Reply = ();

    #[instrument(name = "datawriter", skip_all, fields(guid = %self.writer.get_guid()))]
    async fn handle(
        &mut self,
        msg: DataWriterActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let now = Utc::now().timestamp_millis();
        match msg {
            DataWriterActorMessage::Write { data, instance } => {
                let change = self.writer.new_change(
                    ChangeKind::Alive,
                    Some(data),
                    Some(self.qos.clone()),
                    instance,
                );
                self.writer.add_change(&mut self.effects, change).unwrap();
            }
            DataWriterActorMessage::IncomingMessage { message } => {
                let message = troc_core::Message::deserialize_from(&message).unwrap();

                self.writer.ingest(&mut self.effects, now, message).unwrap()
            }
            DataWriterActorMessage::AddProxy { proxy, wires } => {
                self.output_wires.extend(wires);
                self.writer.add_proxy(proxy.clone());
                self.timer
                    .tell(TimerActorScheduleTickMessage::Writer {
                        delay: 100,
                        target: ctx.actor_ref().clone(),
                    })
                    .await
                    .unwrap();
                let res = self
                    .event_sender
                    .send(DataWriterEvent::SubscriptionMatched(proxy));
            }
            DataWriterActorMessage::RemoveProxy { guid, locators } => {
                for locator in locators {
                    self.output_wires.remove(&locator);
                }
                self.writer.remove_proxy(guid)
            }
            DataWriterActorMessage::Tick => self.writer.tick(&mut self.effects, now),
            DataWriterActorMessage::AddInputWire { wires, locators } => {
                for wire in &wires {
                    wire.tell(ReceiverWireActorMessage::Start {
                        actor_dest: ctx.actor_ref().clone(),
                    })
                    .await
                    .unwrap();
                }
                self.input_wires.extend(wires);
                self.writer.add_unicast_locators(locators);
            }
        }

        while let Some(effect) = self.effects.pop() {
            match effect {
                Effect::Message {
                    timestamp_millis,
                    message,
                    locators,
                } => {
                    let mut buffer = BytesMut::zeroed(65 * 1024);
                    let nb_bytes = message.serialize_to(&mut buffer).unwrap();
                    let message = buffer.split_to(nb_bytes);

                    for locator in locators.iter() {
                        if let Some(actor) = self.output_wires.get(locator) {
                            actor
                                .tell(SenderWireActorMessage {
                                    buffer: message.clone(),
                                })
                                .await
                                .unwrap();
                        }
                    }
                }
                Effect::ScheduleTick { id: _, delay } => {
                    self.timer
                        .tell(TimerActorScheduleTickMessage::Writer {
                            delay: 200,
                            target: ctx.actor_ref().clone(),
                        })
                        .await
                        .unwrap();
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
    pub qos: InlineQos,
    pub timer: ActorRef<TimerActor>,
}

#[derive(Debug)]
pub struct DataWriterActor {
    writer: Writer,
    qos: InlineQos,
    effects: Effects,
    timer: ActorRef<TimerActor>,
    input_wires: Vec<ActorRef<ReceiverWireActor>>,
    output_wires: HashMap<Locator, ActorRef<SenderWireActor>>,
    _event_receiver: Option<Receiver<DataWriterEvent>>,
    event_sender: Sender<DataWriterEvent>,
}

impl Actor for DataWriterActor {
    type Args = DataWriterActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let DataWriterActorCreateObject { writer, qos, timer } = args;

        let (event_sender, event_receiver) = channel(64);

        let datawriter_actor = Self {
            writer,
            qos,
            effects: Effects::default(),
            timer,
            input_wires: Default::default(),
            output_wires: Default::default(),
            _event_receiver: Some(event_receiver),
            event_sender,
        };

        Ok(datawriter_actor)
    }

    async fn on_stop(
        &mut self,
        _actor_ref: kameo::prelude::WeakActorRef<Self>,
        _reason: kameo::prelude::ActorStopReason,
    ) -> Result<(), Self::Error> {
        for wire in self.input_wires.iter() {
            wire.stop_gracefully().await.unwrap();
            wire.wait_for_shutdown().await;
        }
        for wire in self.output_wires.values() {
            wire.stop_gracefully().await.unwrap();
            wire.wait_for_shutdown().await;
        }
        Ok(())
    }
}

impl Sendable for DataWriterActor {
    type Msg = DataWriterActorMessage;

    fn build_message(buffer: BytesMut) -> Self::Msg {
        DataWriterActorMessage::IncomingMessage { message: buffer }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::*;
    use tokio::sync::mpsc::channel;
    use troc_core::{Guid, Locator};

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
