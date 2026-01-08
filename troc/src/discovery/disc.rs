use std::{collections::HashMap, time::Duration};

use bytes::BytesMut;
use chrono::Utc;
use kameo::{Actor, prelude::Message};
use tokio::time::{Instant, sleep};
use tracing::{Level, event};
use troc_core::{
    DdsError, Discovery as ProtocolDiscovery, DiscoveryBuilder, DiscoveryConfiguration, Effect,
    Effects, IncommingMessage, OutcommingMessage, ReaderProxy, WriterProxy,
};

use troc_core::types::{EntityId, InlineQos, ParticipantProxy};

use crate::discovery::DiscoveryEvent;

#[derive(Debug)]
pub enum DiscoveryActorMessage {
    ParticipantProxyChanged(ParticipantProxy),
    WriterCreated {
        writer_proxy: WriterProxy,
    },
    WriterRemoved(EntityId),
    ReaderCreated {
        reader_proxy: ReaderProxy,
    },
    ReaderRemoved(EntityId),
    Tick,
    Message {
        message: troc_core::messages::Message,
    },
}

impl Message<DiscoveryActorMessage> for DiscoveryActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: DiscoveryActorMessage,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let now = Utc::now().timestamp_millis();
        match msg {
            DiscoveryActorMessage::ParticipantProxyChanged(participant_proxy) => {
                self.discovery
                    .update_participant_infos(&mut self.effects, participant_proxy)
                    .unwrap();
            }
            DiscoveryActorMessage::WriterCreated { writer_proxy } => {
                self.discovery
                    .add_publications_infos(&mut self.effects, writer_proxy, InlineQos::default())
                    .unwrap();
            }
            DiscoveryActorMessage::WriterRemoved(entity_id) => {
                self.discovery.remove_publications_infos(entity_id).unwrap();
            }
            DiscoveryActorMessage::ReaderCreated { reader_proxy } => self
                .discovery
                .add_subscriptions_infos(&mut self.effects, reader_proxy, InlineQos::default())
                .unwrap(),
            DiscoveryActorMessage::ReaderRemoved(entity_id) => {
                self.discovery
                    .remove_subscriptions_infos(entity_id)
                    .unwrap();
            }
            DiscoveryActorMessage::Tick => self.discovery.tick(&mut self.effects, now).unwrap(),
            DiscoveryActorMessage::Message { message } => self
                .discovery
                .ingest(&mut self.effects, message, now)
                .unwrap(),
        }

        while let Some(effect) = self.effects.pop() {
            //
        }
    }
}

#[derive()]
pub struct DiscoveryActor {
    discovery: ProtocolDiscovery,
    effects: Effects,
}

impl Actor for DiscoveryActor {
    type Args = ();

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl DiscoveryActor {
    // fn new_args(
    //     participant_infos: ParticipantProxy,
    //     participant_infos_receiver: Receiver<ParticipantProxy>,
    //     endpoint_lifecycle_receiver: Receiver<EndpointLifecycleCommand>,
    // ) -> Self {
    //     let disc = DiscoveryBuilder::new(participant_infos, DiscoveryConfiguration::new()).build();
    //     let disc = Arc::new(Mutex::new(disc));
    //     let cancellation_token = CancellationToken::new();

    //     let myself = Self {
    //         disc,
    //         cancellation_token,
    //     };

    //     let applicative_writer_index =
    //         HashMap::<EntityId, Sender<DataWriterProxyCommand>>::default();
    //     // TODO: same for the Readers

    //     myself.launch(
    //         applicative_writer_index,
    //         participant_infos_receiver,
    //         endpoint_lifecycle_receiver,
    //     );

    //     // TODO: launch input tasks

    //     myself
    //     todo!()
    // }

    // fn launch(
    //     &self,
    //     mut applicative_writer_index: HashMap<EntityId, Sender<DataWriterProxyCommand>>,
    //     mut participant_infos_receiver: Receiver<ParticipantProxy>,
    //     mut endpoint_lifecycle_receiver: Receiver<EndpointLifecycleCommand>,
    // ) {
    //     let disc = self.disc.clone();
    //     let cancellation_token = self.cancellation_token.child_token();
    //     tokio::spawn(async move {
    //         let pdp_announce_sleep = sleep(Duration::from_secs(5));
    //         tokio::pin!(pdp_announce_sleep);
    //         let edp_heartbeat_sleep = sleep(Duration::from_secs(5));
    //         tokio::pin!(edp_heartbeat_sleep);

    //         Self::pdp_announce(&disc).await;
    //         loop {
    //             select! {
    //                 // PDP
    //                 () = &mut pdp_announce_sleep => {
    //                     Self::pdp_announce(&disc).await;
    //                     pdp_announce_sleep.as_mut().reset(Instant::now() + Duration::from_secs(5));
    //                 }
    //                 Some(infos) = participant_infos_receiver.recv() => {
    //                     Self::update_participant_infos(&disc, infos);
    //                 }
    //                 _ = pdp_change_available_adapter.wait() => {
    //                     disc.lock().unwrap().try_associate_participant().unwrap();
    //                 }
    //                 Some((participant_guid_prefix, delay)) = pdp_stale_adapter.wait_participant_refreshed() => {
    //                     let disc = disc.clone();
    //                     tokio::spawn(async move {
    //                         sleep(delay).await;
    //                         disc.lock().unwrap().remove_participant(participant_guid_prefix).unwrap();
    //                     });
    //                 }
    //                 // EDP
    //                 () = &mut edp_heartbeat_sleep => {
    //                     Self::edp_heartbeat(&disc);
    //                     edp_heartbeat_sleep.as_mut().reset(Instant::now() + Duration::from_secs(5));
    //                 }
    //                 Some(command) = endpoint_lifecycle_receiver.recv() => {
    //                     Self::manage_endpoint_lifecycle(&disc, command, &mut applicative_writer_index);
    //                 }
    //                 _ = edp_pub_change_available_adapter.wait() => {
    //                     disc.lock().unwrap().try_associate_publication().unwrap();
    //                 }
    //                 _ = edp_sub_change_available_adapter.wait() => {
    //                     disc.lock().unwrap().try_associate_subscription().unwrap();
    //                 }
    //                 Some(NackedSchedule { reader_guid, delay }) = pub_nacked_schedule_adapter.wait_nacked_schedule() => {
    //                     let disc = disc.clone();
    //                     tokio::spawn(async move {
    //                         sleep(delay).await;
    //                         // let emission_infos_storage = emission_infos_storage.clone();
    //                         let nacked_production_result = disc.lock().unwrap().produce_publications_announcer_nacked_data(reader_guid);
    //                         match nacked_production_result {
    //                             Ok(Some(out_msg)) => {
    //                                 tokio::spawn(async move {
    //                                     // Self::process_outcomming_message(out_msg, emission_infos_storage).await;
    //                                 });
    //                             }
    //                             Ok(None) => {
    //                                 event!(Level::DEBUG, "No missing data")
    //                             }
    //                             Err(e) => {
    //                                 event!(Level::ERROR, "{e}");
    //                             }
    //                         }
    //                     });
    //                 }
    //                 Some(NackedSchedule { reader_guid, delay }) = sub_nacked_schedule_adapter.wait_nacked_schedule() => {
    //                     let disc = disc.clone();
    //                     tokio::spawn(async move {
    //                         sleep(delay).await;
    //                         // let emission_infos_storage = emission_infos_storage.clone();
    //                         let nacked_production_result = disc.lock().unwrap().produce_subscriptions_announcer_nacked_data(reader_guid);
    //                         match nacked_production_result {
    //                             Ok(Some(out_msg)) => {
    //                                 tokio::spawn(async move {
    //                                     // Self::process_outcomming_message(out_msg, emission_infos_storage).await;
    //                                 });
    //                             }
    //                             Ok(None) => {
    //                                 event!(Level::DEBUG, "No missing data")
    //                             }
    //                             Err(e) => {
    //                                 event!(Level::ERROR, "{e}");
    //                             }
    //                         }
    //                     });
    //                 }
    //                 Some(AckSchedule { writer_guid, delay }) = pub_ack_schedule_adapter.wait_ack_schedule() => {
    //                     let disc = disc.clone();
    //                     tokio::spawn(async move {
    //                         sleep(delay).await;
    //                         // let emission_infos_storage = emission_infos_storage.clone();
    //                         let nacked_production_result = disc.lock().unwrap().produce_publications_detector_acknowledgement(writer_guid);
    //                         match nacked_production_result {
    //                             Ok(Some(out_msg)) => {
    //                                 tokio::spawn(async move {
    //                                     // Self::process_outcomming_message(out_msg, emission_infos_storage).await;
    //                                 });
    //                             }
    //                             Ok(None) => {
    //                                 event!(Level::DEBUG, "No missing data")
    //                             }
    //                             Err(e) => {
    //                                 event!(Level::ERROR, "{e}");
    //                             }
    //                         }
    //                     });
    //                 }
    //                 Some(AckSchedule { writer_guid, delay }) = sub_ack_schedule_adapter.wait_ack_schedule() => {
    //                     let disc = disc.clone();
    //                     tokio::spawn(async move {
    //                         sleep(delay).await;
    //                         // let emission_infos_storage = emission_infos_storage.clone();
    //                         let nacked_production_result = disc.lock().unwrap().produce_subscriptions_detector_acknowledgement(writer_guid);
    //                         match nacked_production_result {
    //                             Ok(Some(out_msg)) => {
    //                                 tokio::spawn(async move {
    //                                     // Self::process_outcomming_message(out_msg, emission_infos_storage).await;
    //                                 });
    //                             }
    //                             Ok(None) => {
    //                                 event!(Level::DEBUG, "No missing data")
    //                             }
    //                             Err(e) => {
    //                                 event!(Level::ERROR, "{e}");
    //                             }
    //                         }
    //                     });
    //                 }
    //                 Some(event) = event_receiver_adapter.wait_discovery_event() => {
    //                     Self::manage_discovery_event(event, &mut applicative_writer_index).await;
    //                 }
    //                 _ = cancellation_token.cancelled() => {
    //                     break
    //                 }
    //                 else => {
    //                     event!(Level::ERROR, "Incomming task terminate");
    //                     break;
    //                 }
    //             }
    //         }
    //     });
    // }

    // async fn pdp_input_task(&self, mut wire: Wire) {
    //     let disc = self.disc.clone();
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
    //                                 if let Err(e) = disc.lock().unwrap().process_pdp_incomming_message(message) {
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

    // async fn edp_input_task(&self, mut wire: Wire) {
    //     let disc = self.disc.clone();
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
    //                                 if let Err(e) = disc.lock().unwrap().process_edp_incomming_message(message) {
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

    // async fn pdp_announce(disc: &Arc<Mutex<ProtocolDiscovery>>) {
    //     let announcement_result = disc.lock().unwrap().produce_participant_announce();
    //     match announcement_result {
    //         Ok(out_msg) => {
    //             // TODO:
    //             // - serialize msg
    //             // - send to IO
    //         }
    //         Err(e) => {
    //             event!(Level::ERROR, "{e}");
    //         }
    //     }
    // }

    // fn update_participant_infos(disc: &Arc<Mutex<ProtocolDiscovery>>, infos: ParticipantProxy) {
    //     if let Err(e) = disc.lock().unwrap().update_participant_infos(infos) {
    //         event!(Level::ERROR, "{e}")
    //     }
    // }

    // fn manage_endpoint_lifecycle(
    //     disc: &Arc<Mutex<ProtocolDiscovery>>,
    //     endpoint_life_cmd: EndpointLifecycleCommand,
    //     applicative_writer_index: &mut HashMap<EntityId, Sender<DataWriterProxyCommand>>,
    // ) {
    //     let announce_producation_result = match endpoint_life_cmd {
    //         EndpointLifecycleCommand::WriterCreated {
    //             writer_proxy,
    //             inline_qos,
    //             command_sender,
    //         } => {
    //             applicative_writer_index.insert(
    //                 writer_proxy.get_remote_writer_guid().get_entity_id(),
    //                 command_sender,
    //             );
    //             disc.lock()
    //                 .unwrap()
    //                 .produce_publication_announces(writer_proxy, inline_qos)
    //         }
    //         EndpointLifecycleCommand::ReaderCreated {
    //             reader_proxy,
    //             inline_qos,
    //             command_sender,
    //         } => disc
    //             .lock()
    //             .unwrap()
    //             .produce_subscription_announces(reader_proxy, inline_qos),
    //     };

    //     match announce_producation_result {
    //         Ok(announce) => {
    //             // let emission_infos_storage: EmissionInfosStorage = emission_infos_storage.clone();
    //             // tokio::spawn(async move {
    //             //     Self::process_outcomming_message(announce, emission_infos_storage).await;
    //             // });
    //         }
    //         Err(_) => todo!(),
    //     }
    // }

    // fn edp_heartbeat(disc: &Arc<Mutex<ProtocolDiscovery>>) {
    //     let heartbeat_production_result = disc.lock().unwrap().produce_edp_heartbeat();
    //     match heartbeat_production_result {
    //         Ok(heartbeats) => {
    //             for heartbeat in heartbeats {
    //                 // let emission_infos_storage: EmissionInfosStorage = emission_infos_storage.clone();
    //                 // tokio::spawn(async move {
    //                 //     Self::process_outcomming_message(heartbeat, emission_infos_storage).await;
    //                 // });
    //             }
    //         }
    //         Err(_) => todo!(),
    //     }
    // }

    // async fn manage_discovery_event(
    //     event: DiscoveryEvent,
    //     applicative_writer_index: &mut HashMap<EntityId, Sender<DataWriterProxyCommand>>,
    // ) {
    //     match event {
    //         DiscoveryEvent::WriterDetected { writer_id, proxy } => todo!(),
    //         DiscoveryEvent::WriterMatched { writer_id, proxy } => todo!(),
    //         DiscoveryEvent::WriterRemoved { writer_id } => todo!(),
    //         DiscoveryEvent::ReaderDetected { reader_id, proxy } => todo!(),
    //         DiscoveryEvent::ReaderMatched { reader_id, proxy } => {
    //             if let Some(sender) = applicative_writer_index.get_mut(&reader_id) {
    //                 let proxy_command = DataWriterProxyCommand::AddProxy {
    //                     proxy,
    //                     emission_infos: (),
    //                 };
    //                 sender.send(proxy_command).await.unwrap();
    //             }
    //         }
    //         DiscoveryEvent::ReaderRemoved { reader_id } => todo!(),
    //         DiscoveryEvent::ParticipantDiscovered { proxy } => todo!(),
    //         DiscoveryEvent::ParticipantUpdated { proxy } => todo!(),
    //         DiscoveryEvent::ParticipantRemoved { guid_prefix } => todo!(),
    //     }
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
