use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::BytesMut;
use chrono::Utc;
use kameo::{Actor, actor::ActorRef, prelude::Message};
use serde::Deserialize;
use tokio::{
    runtime::Handle,
    select,
    sync::{
        Notify,
        broadcast::{Receiver, Sender, channel},
    },
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{Level, Span, event};
use troc_core::{
    CacheChangeContainer, DdsError, Effect, IncommingMessage, LocatorList, OutcommingMessage,
    Reader, WriterProxy,
};
use troc_core::{Effects, Keyed};
use troc_core::{Guid, InlineQos, Locator, SerializedData, cdr};

use crate::{
    DataReaderEvent,
    discovery::DiscoveryActor,
    infrastructure::QosPolicy,
    subscription::{
        DataReaderListener, condition::ReadCondition, data_sample::DataSample,
        sample_info::SampleInfo,
    },
    time::{TimerActor, TimerActorScheduleTickMessage},
    wires::{
        ReceiverWireActor, ReceiverWireActorMessage, Sendable, SenderWireActor,
        SenderWireActorMessage,
    },
};

#[derive()]
pub struct DataReader<T> {
    guid: Guid,
    qos: InlineQos,
    data_reader_actor: ActorRef<DataReaderActor>,
    data_availability_notifier: Arc<Notify>,
    phantom: PhantomData<T>,
}

impl<T> DataReader<T> {
    pub(crate) async fn new(
        guid: Guid,
        qos: QosPolicy,
        data_reader_actor: ActorRef<DataReaderActor>,
        data_availability_notifier: Arc<Notify>,
    ) -> Self {
        Self {
            guid,
            qos: qos.into(),
            data_reader_actor,
            data_availability_notifier,
            phantom: PhantomData,
        }
    }

    pub fn get_guid(&self) -> Guid {
        self.guid
    }

    pub async fn get_listener(&self) -> Result<DataReaderListener, DdsError> {
        let receiver = self
            .data_reader_actor
            .ask(DataReaderListenerCreate {})
            .await
            .unwrap();
        Ok(DataReaderListener { receiver })
    }

    pub async fn read_next_sample_raw(&mut self) -> Result<DataSample<SerializedData>, DdsError> {
        loop {
            match self
                .data_reader_actor
                .ask(DataReaderActorReadOneMessage::Read {})
                .await
                .unwrap()
            {
                Some(change) => {
                    let data = change.data.clone();
                    let infos = SampleInfo::from(&change.infos);
                    let sample = DataSample::<SerializedData>::new(infos, data);
                    return Ok(sample);
                }
                None => self.data_availability_notifier.notified().await,
            }
        }
    }

    pub async fn read_next_sample_raw_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<DataSample<SerializedData>, DdsError> {
        match tokio::time::timeout(timeout, self.read_next_sample_raw()).await {
            Ok(res) => res,
            Err(e) => Err(DdsError::Timeout {
                cause: e.to_string(),
            }),
        }
    }

    pub async fn take_next_sample_raw(&mut self) -> Result<DataSample<T>, DdsError> {
        unimplemented!()
    }

    pub async fn read_next_sample(&mut self) -> Result<DataSample<T>, DdsError>
    where
        for<'a> T: Deserialize<'a> + 'static + Keyed,
    {
        let DataSample { infos, data } = self.read_next_sample_raw().await?;
        let data = if let Some(data) = data {
            let deserialized_data = cdr::deserialize(data.get_data()).unwrap();
            Some(deserialized_data)
        } else {
            None
        };
        let typed_sample = DataSample::<T>::new(infos, data);
        Ok(typed_sample)
    }

    pub async fn read_next_sample_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<DataSample<T>, DdsError>
    where
        for<'a> T: Deserialize<'a> + 'static + Keyed,
    {
        match tokio::time::timeout(timeout, self.read_next_sample()).await {
            Ok(res) => res,
            Err(e) => Err(DdsError::Timeout {
                cause: e.to_string(),
            }),
        }
    }

    pub async fn read_next_sample_instance_raw(
        &mut self,
        key: [u8; 16],
    ) -> Result<DataSample<Vec<u8>>, DdsError> {
        unimplemented!()
    }

    pub async fn read_next_sample_instance(
        &mut self,
        key: &impl Keyed,
    ) -> Result<DataSample<T>, DdsError> {
        unimplemented!()
    }

    pub async fn read_next_sample_instance_timeout(
        &mut self,
        key: &impl Keyed,
        timeout: Duration,
    ) -> Result<DataSample<T>, DdsError> {
        unimplemented!()
    }

    pub async fn read(
        &mut self,
        max_samples: usize,
        read_condition: ReadCondition,
    ) -> Result<Vec<DataSample<T>>, DdsError> {
        unimplemented!()
    }

    pub async fn take(
        &mut self,
        max_samples: usize,
        read_condition: ReadCondition,
    ) -> Result<Vec<DataSample<T>>, DdsError> {
        unimplemented!()
    }

    pub async fn read_instance(
        &mut self,
        max_samples: usize,
        read_condition: ReadCondition,
        instance: impl Keyed,
    ) -> Result<Vec<DataSample<T>>, DdsError> {
        unimplemented!()
    }

    pub async fn take_instance(
        &mut self,
        max_samples: usize,
        read_condition: ReadCondition,
        instance: impl Keyed,
    ) -> Result<Vec<DataSample<T>>, DdsError> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum DataReaderActorReadOneMessage {
    Read {
        // TODO
    },
}

impl Message<DataReaderActorReadOneMessage> for DataReaderActor {
    type Reply = Option<CacheChangeContainer>;

    async fn handle(
        &mut self,
        msg: DataReaderActorReadOneMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            DataReaderActorReadOneMessage::Read {} => {
                let a = self.reader.get_first_available_change();
                a.cloned()
            }
        }
    }
}

#[derive(Debug)]
pub struct DataReaderListenerCreate;

impl Message<DataReaderListenerCreate> for DataReaderActor {
    type Reply = Receiver<DataReaderEvent>;

    async fn handle(
        &mut self,
        _msg: DataReaderListenerCreate,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self._event_receiver.take().unwrap()
    }
}

#[derive(Debug)]
pub enum DataReaderActorMessage {
    IncomingMessage {
        message: BytesMut,
    },
    AddProxy {
        proxy: WriterProxy,
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

impl Message<DataReaderActorMessage> for DataReaderActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: DataReaderActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let now = Utc::now().timestamp_millis();
        match msg {
            DataReaderActorMessage::IncomingMessage { message } => {
                let message = troc_core::Message::deserialize_from(&message).unwrap();

                self.reader.ingest(&mut self.effects, now, message).unwrap()
            }
            DataReaderActorMessage::AddProxy { proxy, wires } => {
                self.output_wires.extend(wires);
                self.reader.add_proxy(proxy.clone());
                let res = self
                    .event_sender
                    .send(DataReaderEvent::PublicationMatched(proxy));
            }
            DataReaderActorMessage::RemoveProxy { guid, locators } => {
                for locator in locators {
                    self.output_wires.remove(&locator);
                }
                self.reader.remove_proxy(guid)
            }
            DataReaderActorMessage::Tick => self.reader.tick(&mut self.effects, now),
            DataReaderActorMessage::AddInputWire { wires, locators } => {
                for wire in &wires {
                    wire.tell(ReceiverWireActorMessage::Start {
                        actor_dest: ctx.actor_ref().clone(),
                    })
                    .await
                    .unwrap();
                }
                self.input_wires.extend(wires);
                self.reader.add_unicast_locators(locators);
            }
        }

        while let Some(effect) = self.effects.pop() {
            match effect {
                Effect::DataAvailable => {
                    self.data_availability_notifier.notify_one();
                }
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
                        .tell(TimerActorScheduleTickMessage::Reader {
                            delay,
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
pub struct DataReaderActorCreateObject {
    pub reader: Reader,
    pub qos: InlineQos,
    pub data_availability_notifier: Arc<Notify>,
    pub discovery: ActorRef<DiscoveryActor>,
    pub timer: ActorRef<TimerActor>,
}

#[derive(Debug)]
pub struct DataReaderActor {
    reader: Reader,
    qos: InlineQos,
    effects: Effects,
    discovery: ActorRef<DiscoveryActor>,
    timer: ActorRef<TimerActor>,
    input_wires: Vec<ActorRef<ReceiverWireActor>>,
    output_wires: HashMap<Locator, ActorRef<SenderWireActor>>,
    data_availability_notifier: Arc<Notify>,
    _event_receiver: Option<Receiver<DataReaderEvent>>,
    event_sender: Sender<DataReaderEvent>,
}

impl Actor for DataReaderActor {
    type Args = DataReaderActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let DataReaderActorCreateObject {
            reader,
            qos,
            data_availability_notifier,
            discovery,
            timer,
        } = args;

        let (event_sender, event_receiver) = channel(64);

        let datawriter_actor = Self {
            reader,
            qos,
            effects: Effects::default(),
            discovery,
            timer,
            input_wires: Default::default(),
            output_wires: Default::default(),
            data_availability_notifier,
            _event_receiver: Some(event_receiver),
            event_sender,
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

impl Sendable for DataReaderActor {
    type Msg = DataReaderActorMessage;

    fn build_message(buffer: BytesMut) -> Self::Msg {
        DataReaderActorMessage::IncomingMessage { message: buffer }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        marker::PhantomData,
        str::FromStr,
        sync::{Arc, Mutex},
    };

    use rstest::*;
    use tokio::sync::mpsc::channel;
    use tokio_util::sync::CancellationToken;
    use troc_core::{Guid, InlineQos, Locator};
    use troc_core::{ReaderBuilder, ReaderConfiguration};

    use crate::{domain::Configuration, subscription::datareader::DataReader};

    // #[fixture]
    // fn new() -> DataReader<()> {
    //     let guid = Guid::default();
    //     let qos = InlineQos::default();
    //     let config = ReaderConfiguration::default();
    //     let change_availability_adapter = ChangeAvailabilityAdapter::new();
    //     let (ack_adapter_sender, ack_adapter_receiver) =
    //         AckScheduleAdapterBuilder::create_endpoints();
    //     let reader = ReaderBuilder::new(
    //         guid,
    //         qos.clone(),
    //         Box::new(change_availability_adapter.clone()),
    //         config,
    //     )
    //     .ack_schedule_adapter(ack_adapter_sender)
    //     .build();
    //     // TODO: the 'discovery_command_sender' comes from Discovery in practice
    //     let (discovery_command_sender, discovery_command_receiver) = channel(1024);
    //     let datareader = DataReader {
    //         guid,
    //         qos,
    //         listener: false,
    //         reader: Arc::new(Mutex::new(reader)),
    //         cancellation_token: CancellationToken::new(),
    //         change_availability_adapter,
    //         emission_infos_storage: EmissionInfosStorage::new(),
    //         phantom: PhantomData,
    //     };
    //     datareader
    //         .launch_management_and_timer_task(ack_adapter_receiver, discovery_command_receiver);
    //     let wire_factory = WireFactory::new(0, Configuration::default());
    //     let locator = Locator::from_str("127.0.0.1:65000:UDPV4").unwrap();
    //     let wire = wire_factory
    //         .build_listener_wire_from_locator(&locator)
    //         .unwrap();
    //     datareader.incomming_task(wire);
    //     datareader
    // }
}
