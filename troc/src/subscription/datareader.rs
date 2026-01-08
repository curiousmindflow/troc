use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::BytesMut;
use kameo::{Actor, actor::ActorRef, prelude::Message};
use serde::Deserialize;
use tokio::{
    runtime::Handle,
    select,
    sync::mpsc::{Receiver, Sender},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{Level, Span, event};
use troc_core::{DdsError, Effect, IncommingMessage, OutcommingMessage, Reader, WriterProxy};
use troc_core::{Effects, Keyed};
use troc_core::{
    deserialize_data,
    types::{Guid, InlineQos, Locator, SerializedData},
};

use crate::{
    discovery::DiscoveryActor,
    domain::{DISCOVERY_ACTOR_NAME, TIMER_ACTOR_NAME},
    infrastructure::QosPolicy,
    subscription::{
        DataReaderListener, condition::ReadCondition, data_sample::DataSample,
        sample_info::SampleInfo,
    },
    time::{TimerActor, TimerActorMessage},
    wires::{Wire, WireActor, WireActorMessage},
};

#[derive(Debug)]
pub enum DataReaderProxyCommand {
    AddProxy {
        proxy: WriterProxy,
        emission_infos: Vec<(Locator, Sender<BytesMut>)>,
    },
    RemoveProxy {
        guid: Guid,
        emission_locators: Vec<Locator>,
    },
}

#[derive()]
pub struct DataReader<T> {
    guid: Guid,
    qos: InlineQos,
    data_reader_actor: ActorRef<DataReaderActor>,
    phantom: PhantomData<T>,
}

impl<T> DataReader<T> {
    pub(crate) async fn new(
        guid: Guid,
        qos: QosPolicy,
        data_reader_actor: ActorRef<DataReaderActor>,
    ) -> Self {
        Self {
            guid,
            qos: qos.into(),
            data_reader_actor,
            phantom: PhantomData,
        }
    }

    pub fn get_guid(&self) -> Guid {
        self.guid
    }

    pub async fn get_listener(&self) -> Option<DataReaderListener> {
        unimplemented!()
    }

    pub async fn read_next_sample_raw(&mut self) -> Result<DataSample<SerializedData>, DdsError> {
        // loop {
        //     if let Some(change) = self.reader.lock().unwrap().get_first_available_change() {
        //         let data = change.data.clone();
        //         let infos = SampleInfo::from(&change.infos);
        //         let sample = DataSample::<SerializedData>::new(infos, data);
        //         return Ok(sample);
        //     }
        // }
        todo!()
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
            let deserialized_data = deserialize_data(&data).unwrap();
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

    // fn launch_management_and_timer_task(
    //     &self,
    //     mut ack_schedule_adapter: AckScheduleReceiverAdapter,
    //     mut discovery_command_receiver: Receiver<DataReaderProxyCommand>,
    // ) {
    //     let reader = self.reader.clone();
    //     let emission_infos_storage = self.emission_infos_storage.clone();
    //     let cancellation_token = self.cancellation_token.child_token();
    //     tokio::spawn(async move {
    //         loop {
    //             select! {
    //                     _ = cancellation_token.cancelled() => {
    //                         break
    //                     }
    //                     Some(AckSchedule { writer_guid, delay }) = ack_schedule_adapter.wait_ack_schedule() => {
    //                         let reader = reader.clone();
    //                         let emission_infos_storage = emission_infos_storage.clone();
    //                         tokio::spawn(async move {
    //                             sleep(delay).await;
    //                             {
    //                                 let produce_result = {
    //                                     let mut reader_guard = reader.lock().unwrap();
    //                                     reader_guard.produce_acknowledgment(writer_guid)
    //                                 };
    //                                 match produce_result {
    //                                     Ok(Some(out_msg)) => {
    //                                         Self::process_outcomming_message(out_msg, emission_infos_storage).await;
    //                                     }
    //                                     Ok(None) => {
    //                                         event!(Level::DEBUG, "No missing data")
    //                                     },
    //                                     Err(e) => {
    //                                         event!(Level::ERROR, "{e}");
    //                                     }
    //                                 }
    //                             }
    //                         });
    //                     }
    //                     Some(cmd) = discovery_command_receiver.recv() => {
    //                         Self::manage_proxies(cmd, &reader, &emission_infos_storage).await;
    //                     }
    //                     else => {
    //                         event!(Level::ERROR, "Management task terminate");
    //                         break;
    //                     }
    //             }
    //         }
    //     });
    // }

    // async fn manage_proxies(
    //     cmd: DataReaderProxyCommand,
    //     reader: &Arc<Mutex<Reader>>,
    //     emission_infos_storage: &EmissionInfosStorage,
    // ) {
    //     match cmd {
    //         DataReaderProxyCommand::AddProxy {
    //             proxy,
    //             emission_infos,
    //         } => {
    //             reader.lock().unwrap().add_proxy(proxy);
    //             for (locator, sender) in emission_infos {
    //                 emission_infos_storage.store(locator, sender).await;
    //             }
    //         }
    //         DataReaderProxyCommand::RemoveProxy {
    //             guid,
    //             emission_locators,
    //         } => {
    //             reader.lock().unwrap().remove_proxy(guid);
    //             for locator in emission_locators {
    //                 emission_infos_storage.remove(&locator).await;
    //             }
    //         }
    //     }
    // }

    // fn incomming_task(&self, mut wire: Wire) {
    //     let reader = self.reader.clone();
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
    //                                 if let Err(e) = reader.lock().unwrap().ingest(message) {
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

    // async fn process_outcomming_message(
    //     outcomming_message: OutcommingMessage,
    //     emission_infos_storage: EmissionInfosStorage,
    // ) {
    //     let OutcommingMessage {
    //         timestamp_millis,
    //         message,
    //         locators,
    //     } = outcomming_message;
    //     match tokio::task::spawn_blocking(move || {
    //         let mut emission_buffer = BytesMut::with_capacity(65 * 1024);
    //         match message.serialize_to(&mut emission_buffer) {
    //             Ok(nb_bytes) => Ok(emission_buffer.split_to(nb_bytes)),
    //             Err(e) => Err(e),
    //         }
    //     })
    //     .await
    //     {
    //         Ok(serialization_result) => {
    //             let buffer = match serialization_result {
    //                 Ok(buffer) => buffer,
    //                 Err(e) => {
    //                     event!(Level::ERROR, "{e}");
    //                     return;
    //                 }
    //             };

    //             for locator in locators.iter() {
    //                 emission_infos_storage
    //                     .get(locator)
    //                     .await
    //                     .send(buffer.clone())
    //                     .await
    //                     .unwrap();
    //             }
    //         }
    //         Err(e) => {
    //             event!(Level::ERROR, "{e}");
    //         }
    //     }
    // }
}

#[derive(Debug)]
pub enum DataReaderActorMessage {
    Read {
        // TODO
    },
    Message {
        message: troc_core::messages::Message,
    },
    AddProxy {
        proxy: WriterProxy,
        wires: HashMap<Locator, ActorRef<WireActor>>,
    },
    RemoveProxy {
        guid: Guid,
        locators: Vec<Locator>,
    },
    Tick,
}

impl Message<DataReaderActorMessage> for DataReaderActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: DataReaderActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            DataReaderActorMessage::Read {} => {
                // TODO
            }
            DataReaderActorMessage::Message { message } => {
                self.reader.ingest(&mut self.effects, message).unwrap()
            }
            DataReaderActorMessage::AddProxy { proxy, wires } => {
                self.output_wires.extend(wires);
                self.reader.add_proxy(proxy)
            }
            DataReaderActorMessage::RemoveProxy { guid, locators } => {
                for locator in locators {
                    self.output_wires.remove(&locator);
                }
                self.reader.remove_proxy(guid)
            }
            DataReaderActorMessage::Tick => self.reader.tick(&mut self.effects),
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
                        .tell(TimerActorMessage::ScheduleReaderTick {
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
    pub input_wires: Vec<ActorRef<WireActor>>,
}

#[derive(Debug)]
pub struct DataReaderActor {
    reader: Reader,
    effects: Effects,
    discovery: ActorRef<DiscoveryActor>,
    timer: ActorRef<TimerActor>,
    input_wires: Vec<ActorRef<WireActor>>,
    output_wires: HashMap<Locator, ActorRef<WireActor>>,
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
            input_wires,
        } = args;

        let discovery = ActorRef::<DiscoveryActor>::lookup(DISCOVERY_ACTOR_NAME)
            .unwrap()
            .unwrap();

        let timer = ActorRef::<TimerActor>::lookup(TIMER_ACTOR_NAME)
            .unwrap()
            .unwrap();

        let datawriter_actor = Self {
            reader,
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
    use std::{
        marker::PhantomData,
        str::FromStr,
        sync::{Arc, Mutex},
    };

    use rstest::*;
    use tokio::sync::mpsc::channel;
    use tokio_util::sync::CancellationToken;
    use troc_core::types::{Guid, InlineQos, Locator};
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
