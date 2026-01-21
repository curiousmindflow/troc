use std::collections::HashMap;

use bytes::BytesMut;
use chrono::Utc;
use kameo::actor::ActorRef;
use kameo::{Actor, prelude::Message};
use tokio::sync::broadcast::Sender;
use tracing::{Level, event, instrument, span};
use troc_core::{
    DdsError, Discovery as ProtocolDiscovery, Effect, Effects, GuidPrefix, Locator, ReaderProxy,
    WriterProxy,
};

use troc_core::{EntityId, InlineQos, ParticipantProxy};

use crate::ParticipantEvent;
use crate::time::{TimerActor, TimerActorScheduleTickMessage};
use crate::wires::{
    ReceiverWireActor, ReceiverWireActorMessage, Sendable, SenderWireActor, SenderWireActorMessage,
};

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
    IncomingMessage {
        message: BytesMut,
    },
    AddInputWire {
        wires: HashMap<Locator, ActorRef<ReceiverWireActor>>,
    },
    AddOutputWires {
        wires: HashMap<Locator, ActorRef<SenderWireActor>>,
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
            DiscoveryActorMessage::WriterCreated { writer_proxy } => {
                self.discovery
                    .add_publications_infos(&mut self.effects, writer_proxy, InlineQos::default())
                    .unwrap();
            }
            DiscoveryActorMessage::WriterRemoved(entity_id) => {
                self.discovery.remove_publications_infos(entity_id).unwrap();
            }
            DiscoveryActorMessage::ReaderCreated { reader_proxy } => {
                self.discovery
                    .add_subscriptions_infos(&mut self.effects, reader_proxy, InlineQos::default())
                    .unwrap();
            }
            DiscoveryActorMessage::ReaderRemoved(entity_id) => {
                self.discovery
                    .remove_subscriptions_infos(entity_id)
                    .unwrap();
            }
            DiscoveryActorMessage::Tick => {
                self.discovery.tick(&mut self.effects, now).unwrap();
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
            DiscoveryActorMessage::AddInputWire { wires } => {
                for wire in wires.values() {
                    wire.tell(ReceiverWireActorMessage::Start {
                        actor_dest: ctx.actor_ref().clone(),
                    })
                    .await
                    .unwrap();
                }
                self.input_wires.extend(wires);
                event!(Level::DEBUG, "Input wires received");
            }
            DiscoveryActorMessage::AddOutputWires { wires } => {
                self.output_wires.extend(wires);
                event!(Level::DEBUG, "Output wires received");
            }
        }

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
                        reader_data: local_reader_infos,
                    });
                    event!(
                        Level::DEBUG,
                        success = success,
                        local_reader = local_reader_infos_str,
                        remote_writer = remote_writer_infos_str,
                        "Effect::ReaderMatch processed"
                    );
                }
                Effect::WriterMatch {
                    success,
                    local_writer_infos,
                    remote_reader_infos,
                } => {
                    let local_writer_infos_str = local_writer_infos.to_string();
                    let remote_reader_infos_str = remote_reader_infos.to_string();
                    let _res = self.event_sender.send(ParticipantEvent::WriterDiscovered {
                        writer_data: local_writer_infos,
                    });
                    event!(
                        Level::DEBUG,
                        success = success,
                        local_writer = local_writer_infos_str,
                        remote_reader = remote_reader_infos_str,
                        "Effect::WriterMatch processed"
                    );
                }
                Effect::ScheduleTick { delay } => {
                    self.timer
                        .tell(TimerActorScheduleTickMessage::Discovery {
                            delay,
                            target: ctx.actor_ref().clone(),
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

#[derive(Debug)]
pub struct DiscoveryActorCreateObject {
    pub participant_guid_prefix: GuidPrefix,
    pub discovery: ProtocolDiscovery,
    pub event_sender: Sender<ParticipantEvent>,
    pub timer: ActorRef<TimerActor>,
}

#[derive()]
pub struct DiscoveryActor {
    participant_guid_prefix: GuidPrefix,
    discovery: ProtocolDiscovery,
    effects: Effects,
    timer: ActorRef<TimerActor>,
    event_sender: Sender<ParticipantEvent>,
    input_wires: HashMap<Locator, ActorRef<ReceiverWireActor>>,
    output_wires: HashMap<Locator, ActorRef<SenderWireActor>>,
}

impl Actor for DiscoveryActor {
    type Args = DiscoveryActorCreateObject;

    type Error = DdsError;

    async fn on_start(
        args: Self::Args,
        _actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let actor = Self {
            participant_guid_prefix: args.participant_guid_prefix,
            discovery: args.discovery,
            effects: Effects::default(),
            timer: args.timer,
            event_sender: args.event_sender,
            input_wires: Default::default(),
            output_wires: Default::default(),
        };

        Ok(actor)
    }
}

impl Sendable for DiscoveryActor {
    type Msg = DiscoveryActorMessage;

    fn build_message(buffer: BytesMut) -> Self::Msg {
        DiscoveryActorMessage::IncomingMessage { message: buffer }
    }
}
