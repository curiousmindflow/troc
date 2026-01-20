use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    time::Duration,
};

use crate::{
    LocatorList,
    messages::{Message, Submessage, SubmessageContent},
    types::{
        ChangeKind, ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
        ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
        ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR, EntityId, Guid, GuidPrefix, InlineQos,
        InstanceHandle, ParticipantProxy, PdpDiscoveredParticipantData, ReliabilityKind,
        SerializedData, participant_builtin_topic_data::ParticipantBuiltinTopicData,
    },
};
use binrw::Endian;
use tracing::{Level, event, instrument};

use crate::{
    Reader, ReaderBuilder, ReaderConfiguration, ReaderProxy, Writer, WriterBuilder,
    WriterConfiguration, WriterProxy,
    common::{Effect, Effects, Error, QosPolicyConsistencyChecker},
    discovery::{
        discovered_reader_data::DiscoveredReaderData, discovered_writer_data::DiscoveredWriterData,
    },
    subscription::SampleStateKind,
};

#[derive(Debug)]
pub struct Announce {
    pub data: SerializedData,
    pub inline_qos: InlineQos,
    pub instance: InstanceHandle,
}

#[derive(Debug)]
pub struct DiscoveryConfiguration {
    pub announcement_period: i64,
    pub lease_duration: Duration,
}

impl DiscoveryConfiguration {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for DiscoveryConfiguration {
    fn default() -> Self {
        Self {
            announcement_period: 50000,
            lease_duration: Duration::from_millis(30000),
        }
    }
}

#[derive(Debug)]
struct RemoteParticipantInfos {
    lease_end_time_ms: i64,
    infos: PdpDiscoveredParticipantData,
}

#[derive(Debug)]
struct WriterMatchingInfos {
    disc_data: DiscoveredWriterData,
    matches: HashSet<Guid>,
}

#[derive(Debug)]
struct ReaderMatchingInfos {
    disc_data: DiscoveredReaderData,
    matches: HashSet<Guid>,
}

#[derive()]
pub struct DiscoveryBuilder {
    participant_guid_prefix: GuidPrefix,
    config: DiscoveryConfiguration,
    last_announcement_timestamp_ms: Option<i64>,
}

impl DiscoveryBuilder {
    pub fn new(participant_guid_prefix: GuidPrefix, config: DiscoveryConfiguration) -> Self {
        Self {
            participant_guid_prefix,
            config,
            last_announcement_timestamp_ms: None,
        }
    }

    pub fn last_announcement(mut self, last_announcement: i64) -> Self {
        self.last_announcement_timestamp_ms = Some(last_announcement);
        self
    }

    pub fn build(self) -> Discovery {
        let Self {
            participant_guid_prefix,
            config,
            last_announcement_timestamp_ms,
        } = self;

        let pdp_announcer = WriterBuilder::new(
            Guid::new(
                participant_guid_prefix,
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            ),
            InlineQos::default(),
        )
        .build();

        let pdp_detector = ReaderBuilder::new(
            Guid::new(
                participant_guid_prefix,
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR,
            ),
            InlineQos::default(),
        )
        .build();

        let edp_pub_announcer = WriterBuilder::new(
            Guid::new(
                participant_guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ),
            InlineQos::default(),
        )
        .reliability(ReliabilityKind::Reliable)
        .build();

        let edp_sub_announcer = WriterBuilder::new(
            Guid::new(
                participant_guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ),
            InlineQos::default(),
        )
        .reliability(ReliabilityKind::Reliable)
        .build();

        let edp_pub_detector = ReaderBuilder::new(
            Guid::new(
                participant_guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ),
            InlineQos::default(),
        )
        .reliability(ReliabilityKind::Reliable)
        .build();

        let edp_sub_detector = ReaderBuilder::new(
            Guid::new(
                participant_guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            ),
            InlineQos::default(),
        )
        .reliability(ReliabilityKind::Reliable)
        .build();

        Discovery {
            participant_guid_prefix,
            pdp_announcer,
            pdp_detector,
            edp_pub_announcer,
            edp_pub_detector,
            edp_sub_announcer,
            edp_sub_detector,
            application_writers_infos: Default::default(),
            application_readers_infos: Default::default(),
            local_participant_infos: Default::default(),
            remote_participants_infos: Default::default(),
            config,
            last_announcement_timestamp_ms: last_announcement_timestamp_ms.unwrap_or(0),
        }
    }
}

#[derive(Debug)]
pub struct Discovery {
    participant_guid_prefix: GuidPrefix,
    pdp_announcer: Writer,
    pdp_detector: Reader,
    edp_pub_announcer: Writer,
    edp_pub_detector: Reader,
    edp_sub_announcer: Writer,
    edp_sub_detector: Reader,
    application_writers_infos: HashMap<EntityId, WriterMatchingInfos>,
    application_readers_infos: HashMap<EntityId, ReaderMatchingInfos>,
    local_participant_infos: ParticipantProxy,
    remote_participants_infos: HashMap<GuidPrefix, RemoteParticipantInfos>,
    config: DiscoveryConfiguration,
    last_announcement_timestamp_ms: i64,
}

impl Discovery {
    #[instrument(skip_all, fields())]
    pub fn update_participant_infos(
        &mut self,
        effects: &mut Effects,
        infos: ParticipantProxy,
    ) -> Result<(), Error> {
        self.local_participant_infos = infos;
        self.produce_participant_announce(effects)?;
        Ok(())
    }

    // TODO: when new local Writer is created we should try to associate it with remote Reader infos previously received
    #[instrument(skip_all, fields())]
    pub fn add_publications_infos(
        &mut self,
        effects: &mut Effects,
        writer_proxy: WriterProxy,
        inline_qos: InlineQos,
    ) -> Result<(), Error> {
        let qos = inline_qos.clone();
        let guid = writer_proxy.get_remote_writer_guid();

        let disc_writer_data = DiscoveredWriterData {
            proxy: writer_proxy,
            params: inline_qos,
        };

        let Ok(data) = disc_writer_data.into_serialized_data(Endian::Big) else {
            //
            todo!()
        };

        let key = guid.as_bytes();
        let instance = InstanceHandle(key);

        let change =
            self.edp_pub_announcer
                .new_change(ChangeKind::Alive, Some(data), Some(qos), instance);

        self.edp_pub_announcer.add_change(effects, change).unwrap();

        event!(Level::DEBUG, "Writer discovery data produced");

        Ok(())
    }

    #[instrument(skip_all, fields())]
    pub fn remove_publications_infos(&mut self, entity_id: EntityId) -> Result<&Effects, Error> {
        // let _res = self
        //     .edp_pub_announcer
        //     .remove_change_for_instance(InstanceHandle::default());
        unimplemented!()
    }

    // TODO: when new local Reader is created we should try to associate it with remote Writer infos previously received
    #[instrument(skip_all, fields())]
    pub fn add_subscriptions_infos(
        &mut self,
        effects: &mut Effects,
        reader_proxy: ReaderProxy,
        inline_qos: InlineQos,
    ) -> Result<(), Error> {
        let qos = inline_qos.clone();
        let guid = reader_proxy.get_remote_reader_guid();

        let disc_reader_data = DiscoveredReaderData {
            proxy: reader_proxy,
            params: inline_qos,
        };

        let Ok(data) = disc_reader_data.into_serialized_data(Endian::Big) else {
            //
            todo!()
        };

        let key = guid.as_bytes();
        let instance = InstanceHandle(key);

        let change =
            self.edp_sub_announcer
                .new_change(ChangeKind::Alive, Some(data), Some(qos), instance);

        self.edp_pub_announcer.add_change(effects, change).unwrap();

        event!(Level::DEBUG, "Reader discovery data produced");

        Ok(())
    }

    #[instrument(skip_all, fields())]
    pub fn remove_subscriptions_infos(&mut self, entity_id: EntityId) -> Result<&Effects, Error> {
        unimplemented!()
    }

    #[instrument(skip_all, fields())]
    pub fn ingest(
        &mut self,
        effects: &mut Effects,
        message: Message,
        now_ms: i64,
    ) -> Result<(), Error> {
        for Submessage { content, .. } in &message.submessages {
            match content {
                SubmessageContent::Data {
                    writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
                    ..
                } => {
                    self.pdp_detector.ingest(effects, now_ms, message).unwrap();
                    let Some(container) = self.pdp_detector.get_first_available_change() else {
                        break;
                    };

                    let data = container.get_data().unwrap();
                    let pdp_participant_data =
                        PdpDiscoveredParticipantData::from_serialized_data(data.clone()).unwrap();
                    let lease_duration_ms: std::time::Duration =
                        pdp_participant_data.get_lease_duration().into();
                    // FIXME: potentially wrong downcast
                    let lease_duration_ms = lease_duration_ms.as_millis() as i64;
                    let lease_end_time_ms = now_ms + lease_duration_ms;

                    let remote_participant_proxy = pdp_participant_data.get_proxy();

                    let remote_participant_guid_prefix =
                        pdp_participant_data.get_guid().get_guid_prefix();
                    let infos = RemoteParticipantInfos {
                        lease_end_time_ms,
                        infos: pdp_participant_data,
                    };

                    match self
                        .remote_participants_infos
                        .entry(remote_participant_guid_prefix)
                    {
                        Entry::Occupied(mut occupied_entry) => {
                            *occupied_entry.get_mut() = infos;
                        }
                        Entry::Vacant(vacant_entry) => {
                            let participant_proxy = infos.infos.get_proxy();
                            vacant_entry.insert(infos);
                            self.produce_participant_announce(effects)?;
                            let effect = Effect::ParticipantMatch { participant_proxy };
                            effects.push(effect);
                        }
                    }

                    self.update_edp_endpoints(&remote_participant_proxy);

                    event!(Level::DEBUG, "Remote Participant discovery data processed");

                    break;
                }
                SubmessageContent::Heartbeat {
                    writer_id: ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                    ..
                } => {
                    self.edp_pub_detector
                        .ingest(effects, now_ms, message)
                        .unwrap();
                    break;
                }
                SubmessageContent::Heartbeat {
                    writer_id: ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                    ..
                } => {
                    self.edp_sub_detector
                        .ingest(effects, now_ms, message)
                        .unwrap();
                    break;
                }
                SubmessageContent::AckNack {
                    reader_id: ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                    ..
                } => {
                    self.edp_pub_announcer
                        .ingest(effects, now_ms, message)
                        .unwrap();
                    break;
                }
                SubmessageContent::AckNack {
                    reader_id: ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                    ..
                } => {
                    self.edp_sub_announcer
                        .ingest(effects, now_ms, message)
                        .unwrap();
                    break;
                }
                SubmessageContent::Data {
                    writer_id: ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                    ..
                } => {
                    self.edp_pub_detector
                        .ingest(effects, now_ms, message)
                        .unwrap();
                    break;
                }
                SubmessageContent::Data {
                    writer_id: ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                    ..
                } => {
                    self.edp_sub_detector
                        .ingest(effects, now_ms, message)
                        .unwrap();
                    break;
                }
                _ => {
                    event!(Level::TRACE, "Unexpected submessage");
                    continue;
                }
            }
        }

        let _res = self.associate_readers(effects);
        let _res = self.associate_writers(effects);

        Ok(())
    }

    #[instrument(skip_all, fields())]
    pub fn tick(&mut self, effects: &mut Effects, now_ms: i64) -> Result<(), Error> {
        let announce_diff = now_ms - self.last_announcement_timestamp_ms;
        // dbg!(self.last_announcement_timestamp_ms);
        // dbg!(now_ms);
        // dbg!(announce_diff);
        // dbg!(self.config.announcement_period);
        if announce_diff >= self.config.announcement_period {
            // self.produce_participant_announce(effects)?;
            self.last_announcement_timestamp_ms = now_ms;
            // dbg!(self.last_announcement_timestamp_ms);
        } else {
            event!(
                Level::TRACE, announcement_period = %self.config.announcement_period, last_announce = %announce_diff,
                "Discovery announcement prevented: announcement period duration not yet elapsed"
            );
        }

        // self.remove_participant(effects, now_ms)?;
        self.edp_pub_announcer.tick(effects, now_ms);
        self.edp_sub_announcer.tick(effects, now_ms);
        self.edp_pub_detector.tick(effects, now_ms);
        self.edp_sub_detector.tick(effects, now_ms);

        // effects.push(Effect::ScheduleTick {
        //     delay: self.config.announcement_period,
        // });
        effects.push(Effect::ScheduleTick { delay: 2000 });

        event!(Level::DEBUG, "Discovery ticked");

        Ok(())
    }

    #[instrument(skip_all, fields())]
    pub fn add_pdp_announcer_proxy(
        &mut self,
        unicast_locators: LocatorList,
        multicast_locators: LocatorList,
    ) {
        let pdp_announcer_proxy = ReaderProxy::new(
            Guid::default(),
            EntityId::default(),
            false,
            true,
            unicast_locators,
            multicast_locators,
        );
        self.pdp_announcer.add_proxy(pdp_announcer_proxy);
    }

    #[instrument(skip_all, fields())]
    pub fn add_pdp_detector_proxy(
        &mut self,
        unicast_locators: LocatorList,
        multicast_locators: LocatorList,
    ) {
        let pdp_detector_proxy = WriterProxy::new(
            Guid::default(),
            EntityId::default(),
            59 * 1024,
            unicast_locators,
            multicast_locators,
        );
        self.pdp_detector.add_proxy(pdp_detector_proxy);
    }

    fn produce_participant_announce(&mut self, effects: &mut Effects) -> Result<(), Error> {
        let participant_data = PdpDiscoveredParticipantData::new(
            ParticipantBuiltinTopicData::default(),
            self.local_participant_infos.clone(),
            self.config.lease_duration.into(),
        );

        let data = participant_data.into_serialized_data(Endian::Big).unwrap();

        let key = self.local_participant_infos.get_guid().as_bytes();
        let instance = InstanceHandle(key);
        let inline_qos = InlineQos {
            key_hash: instance,
            ..Default::default()
        };

        let change = self.pdp_announcer.new_change(
            ChangeKind::Alive,
            Some(data),
            Some(inline_qos),
            instance,
        );

        self.pdp_announcer.add_change(effects, change).unwrap();

        event!(Level::DEBUG, "Participant announcement produced");

        Ok(())
    }

    fn remove_participant(&mut self, effects: &mut Effects, now_ms: i64) -> Result<(), Error> {
        let stale_participants = self
            .remote_participants_infos
            .iter()
            .filter(|(_, p)| p.lease_end_time_ms >= now_ms)
            .map(|(p_guid_prefix, _)| *p_guid_prefix)
            .collect::<Vec<_>>();

        for participant_guid_prefix in stale_participants {
            let old = self
                .remote_participants_infos
                .remove(&participant_guid_prefix);
            assert!(old.is_some());

            self.edp_pub_announcer.remove_proxy(Guid::new(
                participant_guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ));
            self.edp_sub_announcer.remove_proxy(Guid::new(
                participant_guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            ));
            self.edp_pub_detector.remove_proxy(Guid::new(
                participant_guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ));
            self.edp_sub_detector.remove_proxy(Guid::new(
                participant_guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ));

            let effect = Effect::ParticipantRemoved {
                participant_proxy: old.unwrap().infos.get_proxy(),
            };
            effects.push(effect);
        }

        Ok(())
    }

    /// Associate local Readers to remote Writers
    fn associate_readers(&self, effects: &mut Effects) -> Result<(), Error> {
        for potential_match in self
            .edp_pub_detector
            .get_all_available_changes(SampleStateKind::Any)
        {
            let Some(data) = potential_match.get_data() else {
                unreachable!("CacheChange Data submessage presence checked at ingestion")
            };

            let Ok(disc_writer_data) = DiscoveredWriterData::from_serialized_data(data.clone())
            else {
                return Err(Error::DeserializationError);
            };

            for (_id, reader_match_infos) in self.application_readers_infos.iter() {
                if let Err(_e) = QosPolicyConsistencyChecker::check(
                    &disc_writer_data.params,
                    &reader_match_infos.disc_data.params,
                ) {
                    let effect = Effect::ReaderMatch {
                        success: false,
                        local_reader_infos: reader_match_infos.disc_data.clone(),
                        remote_writer_infos: disc_writer_data.clone(),
                    };
                    effects.push(effect);
                }

                let effect = Effect::ReaderMatch {
                    success: true,
                    local_reader_infos: reader_match_infos.disc_data.clone(),
                    remote_writer_infos: disc_writer_data.clone(),
                };
                effects.push(effect);
            }
        }

        Ok(())
    }

    /// Associate local Writers to remote Readers
    fn associate_writers(&self, effects: &mut Effects) -> Result<(), Error> {
        for potential_match in self
            .edp_sub_detector
            .get_all_available_changes(SampleStateKind::Any)
        {
            let Some(data) = potential_match.get_data() else {
                unreachable!("CacheChange Data submessage presence checked at ingestion")
            };

            let Ok(disc_reader_data) = DiscoveredReaderData::from_serialized_data(data.clone())
            else {
                return Err(Error::DeserializationError);
            };

            for (_id, writer_match_infos) in self.application_writers_infos.iter() {
                if let Err(_e) = QosPolicyConsistencyChecker::check(
                    &writer_match_infos.disc_data.params,
                    &disc_reader_data.params,
                ) {
                    let effect = Effect::WriterMatch {
                        success: false,
                        local_writer_infos: writer_match_infos.disc_data.clone(),
                        remote_reader_infos: disc_reader_data.clone(),
                    };
                    effects.push(effect);
                }

                let effect = Effect::WriterMatch {
                    success: true,
                    local_writer_infos: writer_match_infos.disc_data.clone(),
                    remote_reader_infos: disc_reader_data.clone(),
                };
                effects.push(effect);
            }
        }

        Ok(())
    }

    fn update_edp_endpoints(&mut self, participant_proxy: &ParticipantProxy) {
        let builtin_endpoints = participant_proxy.get_available_builtin_endpoints();
        let guid_prefix = participant_proxy.get_guid_prefix();

        let has_publication_announcer =
            builtin_endpoints.disc_builtin_endpoint_publications_announcer() == 1;
        let has_publication_detector =
            builtin_endpoints.disc_builtin_endpoint_publications_detector() == 1;
        let has_subscription_announcer =
            builtin_endpoints.disc_builtin_endpoint_subscriptions_announcer() == 1;
        let has_subscription_detector =
            builtin_endpoints.disc_builtin_endpoint_subscriptions_detector() == 1;

        if has_publication_announcer {
            let endpoint = &mut self.edp_pub_detector;

            let remote_participant_guid =
                Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);

            if !endpoint.lookup_proxy(remote_participant_guid) {
                let writer_proxy = WriterProxy::new(
                    remote_participant_guid,
                    Default::default(),
                    Default::default(),
                    participant_proxy.metatraffic_unicast_locator_list.clone(),
                    participant_proxy.metatraffic_multicast_locator_list.clone(),
                );

                endpoint.add_proxy(writer_proxy);

                event!(
                    Level::DEBUG,
                    "Publication detector<{}> discovered Publication announcer<{}>",
                    &endpoint.get_guid(),
                    &remote_participant_guid
                );
            } else {
                event!(
                    Level::TRACE,
                    "Publication detector<{}> already knows: {}",
                    &endpoint.get_guid(),
                    &remote_participant_guid
                );
            }
        }

        if has_publication_detector {
            let endpoint = &mut self.edp_pub_announcer;

            let remote_participant_guid =
                Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);

            if !endpoint.lookup_proxy(remote_participant_guid) {
                let reader_proxy = ReaderProxy::new(
                    remote_participant_guid,
                    Default::default(),
                    true,
                    true,
                    participant_proxy.metatraffic_unicast_locator_list.clone(),
                    participant_proxy.metatraffic_multicast_locator_list.clone(),
                );
                endpoint.add_proxy(reader_proxy);

                event!(
                    Level::DEBUG,
                    "Publication announcer<{}> discovered Publication detector<{}>",
                    &endpoint.get_guid(),
                    &remote_participant_guid
                );
            } else {
                event!(
                    Level::TRACE,
                    "Publication announcer<{}> already knows: {}",
                    &endpoint.get_guid(),
                    &remote_participant_guid
                );
            }
        }

        if has_subscription_announcer {
            let endpoint = &mut self.edp_sub_detector;

            let remote_participant_guid =
                Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);

            if !endpoint.lookup_proxy(remote_participant_guid) {
                let writer_proxy = WriterProxy::new(
                    remote_participant_guid,
                    Default::default(),
                    Default::default(),
                    participant_proxy.metatraffic_unicast_locator_list.clone(),
                    participant_proxy.metatraffic_multicast_locator_list.clone(),
                );
                endpoint.add_proxy(writer_proxy);

                event!(
                    Level::DEBUG,
                    "Subscription detector<{}> discovered Subscription announcer<{}>",
                    &endpoint.get_guid(),
                    &remote_participant_guid
                );
            } else {
                event!(
                    Level::TRACE,
                    "Subscription detector<{}> already knows: {}",
                    &endpoint.get_guid(),
                    &remote_participant_guid
                );
            }
        }

        if has_subscription_detector {
            let endpoint = &mut self.edp_sub_announcer;

            let remote_participant_guid =
                Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);

            if !endpoint.lookup_proxy(remote_participant_guid) {
                let reader_proxy = ReaderProxy::new(
                    remote_participant_guid,
                    Default::default(),
                    true,
                    true,
                    participant_proxy.metatraffic_unicast_locator_list.clone(),
                    participant_proxy.metatraffic_multicast_locator_list.clone(),
                );
                endpoint.add_proxy(reader_proxy);

                event!(
                    Level::DEBUG,
                    "Subscription announcer<{}> discovered Subscription detector<{}>",
                    &endpoint.get_guid(),
                    &remote_participant_guid
                );
            } else {
                event!(
                    Level::TRACE,
                    "Subscription announcer<{}> already knows: {}",
                    &endpoint.get_guid(),
                    &remote_participant_guid
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        messages::MessageFactory,
        types::{
            ContentNature, DomainTag, ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER, Guid,
            GuidPrefix, Locator, LocatorList, ParticipantProxy, PdpDiscoveredParticipantData,
            SequenceNumber, builtin_endpoint_qos::BuiltinEndpointQos,
            builtin_endpoint_set::BuiltinEndpointSet, domain_id::DomainId, duration::Duration,
            participant_builtin_topic_data::ParticipantBuiltinTopicData,
        },
    };
    use binrw::Endian;
    use rstest::{fixture, rstest};

    use crate::{
        Effect, Effects,
        common::tests::setup_guid_prefix,
        discovery::{Discovery, DiscoveryBuilder, DiscoveryConfiguration},
    };

    #[rstest]
    fn announce(#[from(setup_discovery)] mut discovery: Discovery) {
        let mut effects = Effects::new();

        discovery.tick(&mut effects, 1000).unwrap();

        let message_effect =
            effects.find(|current_effect| matches!(current_effect, Effect::Message { .. }));

        assert!(message_effect.is_some());

        let schedule_effect =
            effects.find(|current_effect| matches!(current_effect, Effect::ScheduleTick { .. }));

        assert!(schedule_effect.is_some());
    }

    // TODO: should also check that a message coming from itself (same participant GuidPrefix) is discarded
    #[rstest]
    fn receive_remote_participant_infos(
        #[from(setup_discovery)] mut discovery: Discovery,
        #[from(setup_participant_infos_1)] participant_proxy: ParticipantProxy,
        #[from(setup_guid_prefix)]
        #[with(1)]
        remote_participant_guid_prefix: GuidPrefix,
    ) {
        let mut effects = Effects::new();
        let participant_disc_data = PdpDiscoveredParticipantData::new(
            ParticipantBuiltinTopicData::default(),
            participant_proxy,
            Duration::from(std::time::Duration::from_millis(1000)),
        );
        let data = participant_disc_data
            .into_serialized_data(Endian::Little)
            .unwrap();
        let message = MessageFactory::new(remote_participant_guid_prefix)
            .message()
            .reader(ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER)
            .writer(ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER)
            .data(ContentNature::Data, SequenceNumber(1), None, Some(data))
            .build();
        assert_eq!(discovery.remote_participants_infos.len(), 0);

        discovery.ingest(&mut effects, message, 1000).unwrap();

        // assert_eq!(discovery.remote_participants_infos.len(), 1);

        // let Some(stored_proxy) = discovery
        //     .remote_participants_infos
        //     .get(&participant_disc_data.get_guid().get_guid_prefix())
        // else {
        //     panic!()
        // };
        // assert_eq!(stored_proxy.infos, participant_disc_data);

        // let schedule_effect = effects.find(|e| matches!(e, Effect::ScheduleTick { .. }));
        // assert!(schedule_effect.is_some());
    }

    #[rstest]
    fn match_remote_participant(#[from(setup_discovery)] mut discovery: Discovery) {}

    #[rstest]
    fn remove_remote_participant(#[from(setup_discovery)] mut discovery: Discovery) {}

    #[fixture]
    fn setup_discovery(
        #[from(setup_participant_infos_0)] infos: ParticipantProxy,
        #[from(setup_configuration)] config: DiscoveryConfiguration,
    ) -> Discovery {
        DiscoveryBuilder::new(infos.get_guid_prefix(), config)
            .last_announcement(0)
            .build()
    }

    #[fixture]
    fn setup_participant_infos_0(
        #[from(setup_guid_prefix)] guid_prefix: GuidPrefix,
    ) -> ParticipantProxy {
        ParticipantProxy::new(
            guid_prefix,
            DomainId(0),
            DomainTag::new(""),
            true,
            LocatorList::new(vec![Locator::from_str("127.0.0.1:7400:UDPV4").unwrap()]),
            LocatorList::new(vec![Locator::from_str("239.255.0.1:7420:UDPV4").unwrap()]),
            LocatorList::default(),
            LocatorList::default(),
            BuiltinEndpointSet::default(),
            BuiltinEndpointQos::default(),
        )
    }

    #[fixture]
    fn setup_participant_infos_1(
        #[from(setup_guid_prefix)]
        #[with(1)]
        guid_prefix: GuidPrefix,
    ) -> ParticipantProxy {
        ParticipantProxy::new(
            guid_prefix,
            DomainId(0),
            DomainTag::new(""),
            true,
            LocatorList::new(vec![Locator::from_str("127.0.0.1:7400:UDPV4").unwrap()]),
            LocatorList::new(vec![Locator::from_str("239.255.0.1:7420:UDPV4").unwrap()]),
            LocatorList::default(),
            LocatorList::default(),
            BuiltinEndpointSet::default(),
            BuiltinEndpointQos::default(),
        )
    }

    #[fixture]
    fn setup_configuration() -> DiscoveryConfiguration {
        DiscoveryConfiguration {
            announcement_period: 500,
            ..Default::default()
        }
    }
}
