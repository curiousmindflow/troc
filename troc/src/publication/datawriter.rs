use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    common::EmissionInfosStorage,
    discovery::EndpointLifecycleCommand,
    infrastructure::QosPolicy,
    publication::{
        DataWriterListener, NackedDataScheduleAdapterBuilder,
        writer_adapter::{NackedDataScheduleReceiverAdapter, NackedSchedule},
    },
    wires::Wire,
};
use bytes::BytesMut;
use protocol::{
    DdsError, IncommingMessage, OutcommingMessage, ReaderProxy, Writer, WriterBuilder,
    WriterConfiguration,
};
use protocol::{
    messages::Message,
    serialize_data,
    types::{
        ChangeKind, Guid, InlineQos, InstanceHandle, Locator, ReliabilityQosPolicy, SequenceNumber,
        SerializedData,
    },
};
use serde::Serialize;
use tokio::{
    runtime::Handle,
    select,
    sync::mpsc::{Receiver, Sender, channel},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{Level, event};
use troc_key::Keyed;

#[derive(Debug)]
pub enum DataWriterProxyCommand {
    AddProxy {
        proxy: ReaderProxy,
        emission_infos: Vec<(Locator, Sender<BytesMut>)>,
    },
    RemoveProxy {
        guid: Guid,
        emission_locators: Vec<Locator>,
    },
}

#[derive()]
pub struct DataWriter<T> {
    guid: Guid,
    qos: InlineQos,
    listener: bool,
    endpoint_lifecycle_sender: Sender<EndpointLifecycleCommand>,
    writer: Arc<Mutex<Writer>>,
    emission_infos_storage: Arc<tokio::sync::Mutex<EmissionInfosStorage>>,
    cancellation_token: CancellationToken,
    phantom: PhantomData<T>,
}

impl<T> DataWriter<T> {
    pub(crate) async fn new(
        guid: Guid,
        qos: QosPolicy,
        endpoint_lifecycle_sender: Sender<EndpointLifecycleCommand>,
    ) -> Self {
        let config = WriterConfiguration::default();
        let (nacked_adapter_sender, nacked_adapter_receiver) =
            NackedDataScheduleAdapterBuilder::create_endpoints();
        let writer = WriterBuilder::new(guid, qos.into(), config)
            .nacked_data_schedule_adapter(nacked_adapter_sender)
            .build();
        let writer_proxy = writer.extract_proxy();
        let emission_infos_storage = Arc::new(Mutex::new(EmissionInfosStorage::new()));

        let data_writer = DataWriter {
            guid,
            qos: qos.into(),
            listener: false,
            endpoint_lifecycle_sender,
            writer: Arc::new(Mutex::new(writer)),
            emission_infos_storage,
            cancellation_token: CancellationToken::new(),
            phantom: PhantomData,
        };

        let (discovery_command_sender, discovery_command_receiver) =
            channel::<DataWriterProxyCommand>(1024);

        data_writer
            .endpoint_lifecycle_sender
            .send(EndpointLifecycleCommand::WriterCreated {
                writer_proxy,
                inline_qos: qos.into(),
                command_sender: discovery_command_sender,
            })
            .await
            .unwrap();

        data_writer.launch_management_and_timer_task(
            Duration::from_millis(500),
            nacked_adapter_receiver,
            discovery_command_receiver,
        );

        data_writer
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
        let mut writer_guard = self.writer.lock().unwrap();
        let change =
            writer_guard.new_change(ChangeKind::Alive, Some(data), Some(self.qos.clone()), key);
        let sequence_number = change.get_sequence_number();
        match writer_guard.add_change(change) {
            Ok(Some(out_msg)) => {
                let emission_infos_storage = self.emission_infos_storage.clone();
                tokio::spawn(async move {
                    Self::process_outcomming_message(out_msg, emission_infos_storage).await;
                });
                Ok(sequence_number)
            }
            Ok(None) => Ok(sequence_number),
            Err(_) => todo!(),
        }
    }

    // TODO: should not be used
    // instead leverages the QoS
    pub async fn remove_sample(&self, sequence: SequenceNumber) {
        unimplemented!()
    }

    pub fn launch_management_and_timer_task(
        &self,
        heartbeat_duration: Duration,
        mut nacked_schedule_adapter: NackedDataScheduleReceiverAdapter,
        mut discovery_command_receiver: Receiver<DataWriterProxyCommand>,
    ) {
        let writer = self.writer.clone();
        let emission_infos_storage = self.emission_infos_storage.clone();
        let cancellation_token = self.cancellation_token.child_token();
        let is_reliable = matches!(self.qos.reliability, ReliabilityQosPolicy::Reliable { .. });
        tokio::spawn(async move {
            let heartbeat_sleep = sleep(heartbeat_duration);
            tokio::pin!(heartbeat_sleep);

            loop {
                select! {
                        _ = cancellation_token.cancelled() => {
                            break
                        }
                        () = &mut heartbeat_sleep, if is_reliable => {
                            let heartbeat_production_result = writer.lock().unwrap().produce_heartbeat();
                            match heartbeat_production_result {
                                Ok(heartbeats) => {
                                    for heartbeat in heartbeats {
                                        let emission_infos_storage = emission_infos_storage.clone();
                                        tokio::spawn(async move {
                                            Self::process_outcomming_message(heartbeat, emission_infos_storage).await;
                                        });
                                    }
                                }
                                Err(e) => {
                                    event!(Level::ERROR, "{e}");
                                }
                            }
                            heartbeat_sleep.as_mut().reset((Instant::now() + heartbeat_duration).into());
                        },
                        Some(NackedSchedule { reader_guid, delay }) = nacked_schedule_adapter.wait_nacked_schedule() => {
                            let writer = writer.clone();
                            let emission_infos_storage = emission_infos_storage.clone();
                            tokio::spawn(async move {
                                sleep(delay).await;
                                let nacked_production_result = writer.lock().unwrap().produce_nacked_data(reader_guid);
                                match nacked_production_result {
                                    Ok(Some(out_msg)) => {
                                        Self::process_outcomming_message(out_msg, emission_infos_storage).await;
                                    }
                                    Ok(None) => {
                                        event!(Level::DEBUG, "No missing data")
                                    }
                                    Err(e) => {
                                        event!(Level::ERROR, "{e}");
                                    }
                                }
                            });
                        }
                        Some(cmd) = discovery_command_receiver.recv() => {
                            Self::manage_proxies(cmd, &writer, &emission_infos_storage).await;
                        }
                        else => {
                            // TODO: trace error event
                            todo!()
                        }
                }
            }
        });
    }

    async fn manage_proxies(
        cmd: DataWriterProxyCommand,
        writer: &Arc<Mutex<Writer>>,
        emission_infos_storage: &EmissionInfosStorage,
    ) {
        match cmd {
            DataWriterProxyCommand::AddProxy {
                proxy,
                emission_infos,
            } => {
                writer.lock().unwrap().add_proxy(proxy);
                for (locator, sender) in emission_infos {
                    emission_infos_storage.store(locator, sender);
                }
            }
            DataWriterProxyCommand::RemoveProxy {
                guid,
                emission_locators,
            } => {
                writer.lock().unwrap().remove_proxy(guid);
                for locator in emission_locators {
                    emission_infos_storage.remove(&locator);
                }
            }
        }
    }

    async fn process_outcomming_message(
        outcomming_message: OutcommingMessage,
        emission_infos_storage: &Arc<tokio::sync::Mutex<EmissionInfosStorage>>,
    ) {
        let OutcommingMessage {
            timestamp_millis,
            message,
            locators,
        } = outcomming_message;
        match tokio::task::spawn_blocking(move || {
            // TODO: use a Memory pool to avoid creating a buffer each time deserialization ocurrs
            let mut emission_buffer = BytesMut::with_capacity(65 * 1024);
            match message.serialize_to(&mut emission_buffer) {
                Ok(nb_bytes) => Ok(emission_buffer.split_to(nb_bytes)),
                Err(e) => Err(e),
            }
        })
        .await
        {
            Ok(serialization_result) => {
                let buffer = match serialization_result {
                    Ok(buffer) => buffer,
                    Err(e) => {
                        event!(Level::ERROR, "{e}");
                        return;
                    }
                };

                for locator in locators.iter() {
                    emission_infos_storage
                        .lock()
                        .await
                        .get(locator)
                        .send(buffer.clone())
                        .await
                        .unwrap();
                }
            }
            Err(e) => {
                event!(Level::ERROR, "{e}");
            }
        }
    }

    pub fn incomming_task(&self, mut wire: Wire) {
        let writer = self.writer.clone();
        let cancellation_token = self.cancellation_token.child_token();
        tokio::spawn(async move {
            loop {
                select! {
                        _ = cancellation_token.cancelled() => {
                            break
                        }
                        Ok(buffer) = wire.recv() => {
                            let Ok(deserialization_result) = Handle::current()
                                .spawn_blocking(move || Message::deserialize_from(&buffer))
                                .await
                            else {
                                event!(Level::ERROR, "Spawn blocking failed");
                                break;
                            };
                            match deserialization_result {
                                Ok(message) => {
                                    let message = IncommingMessage::new(message);
                                    if let Err(e) = writer.lock().unwrap().ingest(message) {
                                        event!(Level::ERROR, "{e}");
                                    }
                                }
                                Err(e) => {
                                    event!(Level::ERROR, "{e}");
                                }
                            }
                        }
                        else => {
                            event!(Level::ERROR, "Incomming task terminate");
                            break;
                        }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::types::{Guid, Locator};
    use rstest::*;
    use tokio::sync::mpsc::channel;

    use crate::{
        domain::Configuration, infrastructure::QosPolicy, publication::datawriter::DataWriter,
        wires::WireFactory,
    };

    #[fixture]
    async fn new() -> DataWriter<()> {
        let guid = Guid::default();
        let qos = QosPolicy::default();

        let (endpoint_lifecycle_sender, _endpoint_lifecycle_receiver) = channel(1024);
        let datawriter = DataWriter::new(guid, qos, endpoint_lifecycle_sender).await;

        let wire_factory = WireFactory::new(0, Configuration::default());
        let locator = Locator::from_str("127.0.0.1:65000:UDPV4").unwrap();
        let wire = wire_factory
            .build_listener_wire_from_locator(&locator)
            .unwrap();
        datawriter.incomming_task(wire);
        datawriter
    }
}
