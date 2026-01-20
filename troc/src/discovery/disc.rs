use std::collections::HashMap;

use bytes::BytesMut;
use chrono::Utc;
use kameo::actor::ActorRef;
use kameo::{Actor, prelude::Message};
use tokio::sync::broadcast::Sender;
use tracing::{Level, event, instrument};
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
                // TODO: here, we get the Locators inside participant proxy, we can then ask WireFactory the Locators we don't have yet
                // since the DiscoveryActor is obviously already created at this point, it's not an issue to pass a clone to the ReceiverWire
                event!(Level::TRACE, "Participant proxy changed",);
                self.discovery
                    .update_participant_infos(&mut self.effects, participant_proxy)
                    .unwrap();
            }
            DiscoveryActorMessage::WriterCreated { writer_proxy } => {
                event!(Level::TRACE, "DataWriter created");
                self.discovery
                    .add_publications_infos(&mut self.effects, writer_proxy, InlineQos::default())
                    .unwrap();
            }
            DiscoveryActorMessage::WriterRemoved(entity_id) => {
                self.discovery.remove_publications_infos(entity_id).unwrap();
            }
            DiscoveryActorMessage::ReaderCreated { reader_proxy } => {
                event!(Level::TRACE, "DataReader created");
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
                event!(Level::TRACE, "Discovery ticked");
                self.discovery.tick(&mut self.effects, now).unwrap()
            }
            DiscoveryActorMessage::IncomingMessage { message } => {
                event!(Level::TRACE, "Incoming message arrived");
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
                event!(Level::TRACE, "Input wires received");
                for wire in wires.values() {
                    wire.tell(ReceiverWireActorMessage::Start {
                        actor_dest: ctx.actor_ref().clone(),
                    })
                    .await
                    .unwrap();
                }
                self.input_wires.extend(wires);
            }
            DiscoveryActorMessage::AddOutputWires { wires } => {
                event!(Level::TRACE, "Output wires received");
                self.output_wires.extend(wires);
            }
        }

        while let Some(effect) = self.effects.pop() {
            match effect {
                Effect::Message {
                    timestamp_millis,
                    message,
                    locators,
                } => {
                    event!(Level::TRACE, ?locators, ?self.output_wires, "Processing Effect::Message");
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

                    event!(Level::DEBUG, "All messages sent");
                }
                Effect::ParticipantMatch { participant_proxy } => {
                    event!(Level::TRACE, "Processing Effect::ParticipantMatch");
                    let _res = self
                        .event_sender
                        .send(ParticipantEvent::ParticipantDiscovered { participant_proxy });
                }
                Effect::ParticipantRemoved { participant_proxy } => {
                    event!(Level::TRACE, "Processing Effect::ParticipantRemoved");
                    let _res = self
                        .event_sender
                        .send(ParticipantEvent::ParticipantRemoved { participant_proxy });
                }
                Effect::ReaderMatch {
                    success,
                    local_reader_infos,
                    remote_writer_infos,
                } => {
                    event!(Level::TRACE, "Processing Effect::ReaderMatch");
                    let _res = self.event_sender.send(ParticipantEvent::ReaderDiscovered {
                        reader_data: local_reader_infos,
                    });
                }
                Effect::WriterMatch {
                    success,
                    local_writer_infos,
                    remote_reader_infos,
                } => {
                    event!(Level::TRACE, "Processing Effect::WriterMatch");
                    let _res = self.event_sender.send(ParticipantEvent::WriterDiscovered {
                        writer_data: local_writer_infos,
                    });
                }
                Effect::ScheduleTick { delay } => {
                    event!(Level::TRACE, delay = %delay, "Processing Effect::ScheduleTick");
                    self.timer
                        .tell(TimerActorScheduleTickMessage::Discovery {
                            delay,
                            target: ctx.actor_ref().clone(),
                        })
                        .await
                        .unwrap();
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
