use std::collections::HashMap;

use bytes::BytesMut;
use chrono::Utc;
use kameo::actor::ActorRef;
use kameo::{Actor, prelude::Message};
use tokio::sync::broadcast::Sender;
use tracing::{Level, event, instrument, span};
use troc_core::{
    DdsError, DiscoveredReaderData, DiscoveredWriterData, Discovery as ProtocolDiscovery,
    DiscoveryBuilder, DiscoveryConfiguration, Effect, Effects, GuidPrefix, Locator, TickId,
};

use troc_core::{EntityId, ParticipantProxy};

use crate::ParticipantEvent;
use crate::publication::{DataWriterActor, DataWriterActorMessage};
use crate::subscription::{DataReaderActor, DataReaderActorMessage};
use crate::time::{TimerActor, TimerActorScheduleTickMessage};
use crate::wires::{
    ReceiverWireActor, ReceiverWireActorMessage, Sendable, SenderWireActor, SenderWireActorMessage,
    SenderWireFactoryActorMessage, WireFactoryActor,
};

#[derive(Debug)]
pub enum DiscoveryActorMessage {
    ParticipantProxyChanged(ParticipantProxy),
    WriterCreated {
        writer_idscovery_data: DiscoveredWriterData,
        actor: ActorRef<DataWriterActor>,
    },
    WriterRemoved(EntityId),
    ReaderCreated {
        reader_discovery_data: DiscoveredReaderData,
        actor: ActorRef<DataReaderActor>,
    },
    ReaderRemoved(EntityId),
    Tick(TickId),
    IncomingMessage {
        message: BytesMut,
    },
}

impl Message<DiscoveryActorMessage> for DiscoveryActor {
    type Reply = ();

    #[instrument(name = "discovery", skip_all, fields(participant_guid_prefix = %self.participant_guid_prefix))]
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
            DiscoveryActorMessage::WriterCreated {
                writer_idscovery_data,
                actor,
            } => {
                self.local_writers.insert(
                    writer_idscovery_data
                        .proxy
                        .get_remote_writer_guid()
                        .get_entity_id(),
                    actor,
                );
                self.discovery
                    .add_publications_infos(&mut self.effects, writer_idscovery_data)
                    .unwrap();
            }
            DiscoveryActorMessage::WriterRemoved(entity_id) => {
                self.discovery.remove_publications_infos(entity_id).unwrap();
            }
            DiscoveryActorMessage::ReaderCreated {
                reader_discovery_data,
                actor,
            } => {
                self.local_readers.insert(
                    reader_discovery_data
                        .proxy
                        .get_remote_reader_guid()
                        .get_entity_id(),
                    actor,
                );
                self.discovery
                    .add_subscriptions_infos(&mut self.effects, reader_discovery_data)
                    .unwrap();
            }
            DiscoveryActorMessage::ReaderRemoved(entity_id) => {
                self.discovery
                    .remove_subscriptions_infos(entity_id)
                    .unwrap();
            }
            DiscoveryActorMessage::Tick(id) => {
                self.discovery.tick(&mut self.effects, now, id).unwrap();
            }
            DiscoveryActorMessage::IncomingMessage { message } => {
                let Ok(message) = tokio::task::spawn_blocking(move || {
                    troc_core::Message::deserialize_from(&message)
                })
                .await
                .unwrap() else {
                    event!(Level::ERROR, "Deserialization failed");
                    return;
                };
                self.discovery
                    .ingest(&mut self.effects, message, now)
                    .unwrap();
            }
        }

        self.process_effects(ctx.actor_ref().clone()).await;
    }
}

#[derive(Debug)]
pub struct DiscoveryActorCreateObject {
    pub participant_proxy: ParticipantProxy,
    pub discovery_configuration: DiscoveryConfiguration,
    pub input_wires: HashMap<Locator, ActorRef<ReceiverWireActor>>,
    pub output_wires: HashMap<Locator, ActorRef<SenderWireActor>>,
    pub event_sender: Sender<ParticipantEvent>,
    pub timer: ActorRef<TimerActor>,
    pub wire_factory: ActorRef<WireFactoryActor>,
}

#[derive()]
pub struct DiscoveryActor {
    participant_guid_prefix: GuidPrefix,
    discovery: ProtocolDiscovery,
    effects: Effects,
    timer: ActorRef<TimerActor>,
    wire_factory: ActorRef<WireFactoryActor>,
    event_sender: Sender<ParticipantEvent>,
    _input_wires: HashMap<Locator, ActorRef<ReceiverWireActor>>,
    output_wires: HashMap<Locator, ActorRef<SenderWireActor>>,
    local_readers: HashMap<EntityId, ActorRef<DataReaderActor>>,
    local_writers: HashMap<EntityId, ActorRef<DataWriterActor>>,
}

impl Actor for DiscoveryActor {
    type Args = DiscoveryActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let DiscoveryActorCreateObject {
            participant_proxy,
            discovery_configuration,
            input_wires,
            output_wires,
            event_sender,
            timer,
            wire_factory,
        } = args;

        let participant_guid_prefix = participant_proxy.get_guid_prefix();

        let mut discovery =
            DiscoveryBuilder::new(participant_guid_prefix, discovery_configuration).build();

        for wire in input_wires.values() {
            wire.tell(ReceiverWireActorMessage::Start {
                actor_dest: actor_ref.clone(),
            })
            .await
            .unwrap();
        }

        let mut effects = Effects::new();

        discovery.init(&mut effects, participant_proxy).unwrap();

        let mut actor = Self {
            participant_guid_prefix,
            discovery,
            effects,
            timer,
            wire_factory,
            event_sender,
            _input_wires: input_wires,
            output_wires,
            local_readers: Default::default(),
            local_writers: Default::default(),
        };

        actor.process_effects(actor_ref.clone()).await;

        Ok(actor)
    }
}

impl DiscoveryActor {
    async fn process_effects(&mut self, actor_ref: kameo::prelude::ActorRef<Self>) {
        while let Some(effect) = self.effects.pop() {
            match effect {
                Effect::Message {
                    timestamp_millis,
                    message,
                    locators,
                } => {
                    let span = span!(Level::TRACE, "message", ?locators, output_wires = ?self.output_wires);
                    let _enter = span.enter();
                    let message = tokio::task::spawn_blocking(move || {
                        // TODO: use a Memory pool to avoid creating a buffer each time serialization ocurrs
                        // FIXME: line above lead to crash because of "failed to fill whole buffer", crash happend at serialization, one line below
                        // let mut emission_buffer = BytesMut::with_capacity(65 * 1024);
                        let mut emission_buffer = BytesMut::zeroed(65 * 1024);
                        let nb_bytes = message.serialize_to(&mut emission_buffer).unwrap();
                        emission_buffer.split_to(nb_bytes)
                    })
                    .await
                    .unwrap();
                    for locator in locators.iter() {
                        if let Some(actor) = self.output_wires.get(locator) {
                            event!(Level::TRACE, "Message sent to {locator}");
                            actor
                                .tell(SenderWireActorMessage {
                                    buffer: message.clone(),
                                })
                                .await
                                .unwrap();
                        }
                    }

                    event!(Level::DEBUG, "Effect::Message processed");
                }
                Effect::ParticipantMatch { participant_proxy } => {
                    let participant_proxy_str = participant_proxy.to_string();
                    let _res = self
                        .event_sender
                        .send(ParticipantEvent::ParticipantDiscovered { participant_proxy });
                    event!(
                        Level::DEBUG,
                        participant_proxy = participant_proxy_str,
                        "Effect::ParticipantMatch processed"
                    );
                }
                Effect::ParticipantRemoved { participant_proxy } => {
                    let participant_proxy_str = participant_proxy.to_string();
                    let _res = self
                        .event_sender
                        .send(ParticipantEvent::ParticipantRemoved { participant_proxy });
                    event!(
                        Level::DEBUG,
                        participant_proxy = participant_proxy_str,
                        "Effect::ParticipantRemoved processed"
                    );
                }
                Effect::ReaderMatch {
                    success,
                    local_reader_infos,
                    remote_writer_infos,
                } => {
                    let local_reader_infos_str = local_reader_infos.to_string();
                    let remote_writer_infos_str = remote_writer_infos.to_string();
                    let _res = self.event_sender.send(ParticipantEvent::ReaderDiscovered {
                        reader_data: local_reader_infos.clone(),
                    });
                    event!(
                        Level::DEBUG,
                        success = success,
                        local_reader = local_reader_infos_str,
                        remote_writer = remote_writer_infos_str,
                        "Effect::ReaderMatch processed"
                    );
                    if success {
                        let local_reader = self
                            .local_readers
                            .get(
                                &local_reader_infos
                                    .proxy
                                    .get_remote_reader_guid()
                                    .get_entity_id(),
                            )
                            .unwrap();

                        let mut output_wires = Vec::default();
                        let (sender_many_to_many, sender_locators_many_to_many) = self
                            .wire_factory
                            .ask(SenderWireFactoryActorMessage::FromLocators {
                                locators: remote_writer_infos.proxy.get_locators(),
                            })
                            .await
                            .unwrap();
                        output_wires.extend(sender_many_to_many);

                        let output_wires: HashMap<Locator, ActorRef<SenderWireActor>> =
                            HashMap::from_iter(
                                sender_locators_many_to_many
                                    .iter()
                                    .zip(output_wires)
                                    .map(|(a, b)| (*a, b)),
                            );

                        local_reader
                            .tell(DataReaderActorMessage::AddProxy {
                                proxy: remote_writer_infos.proxy,
                                wires: output_wires,
                            })
                            .await
                            .unwrap();
                    }
                }
                Effect::WriterMatch {
                    success,
                    local_writer_infos,
                    remote_reader_infos,
                } => {
                    let local_writer_infos_str = local_writer_infos.to_string();
                    let remote_reader_infos_str = remote_reader_infos.to_string();
                    let _res = self.event_sender.send(ParticipantEvent::WriterDiscovered {
                        writer_data: local_writer_infos.clone(),
                    });
                    event!(
                        Level::DEBUG,
                        success = success,
                        local_writer = local_writer_infos_str,
                        remote_reader = remote_reader_infos_str,
                        "Effect::WriterMatch processed"
                    );
                    if success {
                        let local_writer = self
                            .local_writers
                            .get(
                                &local_writer_infos
                                    .proxy
                                    .get_remote_writer_guid()
                                    .get_entity_id(),
                            )
                            .unwrap();

                        let mut output_wires = Vec::default();
                        let (sender_many_to_many, sender_locators_many_to_many) = self
                            .wire_factory
                            .ask(SenderWireFactoryActorMessage::FromLocators {
                                locators: remote_reader_infos.proxy.get_locators(),
                            })
                            .await
                            .unwrap();
                        output_wires.extend(sender_many_to_many);

                        let output_wires: HashMap<Locator, ActorRef<SenderWireActor>> =
                            HashMap::from_iter(
                                sender_locators_many_to_many
                                    .iter()
                                    .zip(output_wires)
                                    .map(|(a, b)| (*a, b)),
                            );

                        local_writer
                            .tell(DataWriterActorMessage::AddProxy {
                                proxy: remote_reader_infos.proxy,
                                wires: output_wires,
                            })
                            .await
                            .unwrap();
                    }
                }
                Effect::ScheduleTick { id, delay } => {
                    self.timer
                        .tell(TimerActorScheduleTickMessage::Discovery {
                            delay,
                            target: actor_ref.clone(),
                            id,
                        })
                        .await
                        .unwrap();
                    event!(Level::DEBUG, delay = %delay, "Effect::ScheduleTick processed");
                }
                _ => continue,
            }
        }
    }
}

impl Sendable for DiscoveryActor {
    type Msg = DiscoveryActorMessage;

    fn build_message(buffer: BytesMut) -> Self::Msg {
        DiscoveryActorMessage::IncomingMessage { message: buffer }
    }
}
