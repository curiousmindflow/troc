use std::{collections::HashMap, fmt::Debug};

use crate::{
    ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER, LocatorList, ReaderProxy,
    messages::{
        GapGroupInfo, HeartbeatGroupInfo, Message, MessageFactory, MessageReceiver,
        SubmessageContent,
    },
    types::{
        ChangeCount, ChangeKind, ContentNature, Count, ENTITYID_UNKOWN, EntityId, FragmentNumber,
        FragmentNumberSet, Guid, HistoryQosPolicy, InlineQos, ReliabilityKind, SequenceNumber,
        SequenceNumberSet, SerializedData, SubmessageFlags,
    },
};
use chrono::Utc;
use itertools::Itertools;
use tracing::{Level, event, instrument};

use crate::{
    CacheChange,
    common::{CacheChangeContainer, Effect, Effects, Error, FragmentedCacheChange, WriterProxy},
    subscription::{
        ReaderHistoryCache, SampleStateKind, historycache::ReaderHistoryCacheConfiguration,
    },
};

#[derive(Debug, Default)]
pub struct ReaderConfiguration {
    heartbeat_response_delay_ms: i64,
    heartbeat_suppression_delay_ms: i64,
}

#[derive(Default)]
pub struct ReaderBuilder {
    guid: Guid,
    // TODO: maybe it's better not to rely on InlineQos here, but just the necessary params
    qos: InlineQos,
    reliability: ReliabilityKind,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
    config: ReaderConfiguration,
}

impl ReaderBuilder {
    pub fn new(guid: Guid, qos: InlineQos) -> Self {
        Self {
            guid,
            qos,
            reliability: ReliabilityKind::BestEffort,
            ..Default::default()
        }
    }

    pub fn reliability(mut self, reliability: ReliabilityKind) -> Self {
        self.reliability = reliability;
        self
    }

    pub fn with_configuration(mut self, config: ReaderConfiguration) -> Self {
        self.config = config;
        self
    }

    pub fn with_unicast_locators(mut self, locators: LocatorList) -> Self {
        self.unicast_locator_list = locators;
        self
    }

    pub fn with_multicast_locators(mut self, locators: LocatorList) -> Self {
        self.multicast_locator_list = locators;
        self
    }

    pub fn build(self) -> Reader {
        let ReaderBuilder {
            guid,
            qos,
            reliability,
            unicast_locator_list,
            multicast_locator_list,
            config,
        } = self;
        let matched_writers = Default::default();
        let depth = match &qos.history {
            HistoryQosPolicy::KeepAll => None,
            HistoryQosPolicy::KeepLast { depth } => Some(*depth),
        };
        let cache_config = ReaderHistoryCacheConfiguration::new(depth, 100, 100, 100);
        let cache = ReaderHistoryCache::new(cache_config);
        let receiver = MessageReceiver::new(self.guid.get_guid_prefix());
        let htb_count = Default::default();
        let htbfrag_count = Default::default();
        let message_factory = MessageFactory::new(self.guid.get_guid_prefix());

        Reader {
            guid,
            matched_writers,
            cache,
            receiver,
            htb_count,
            htbfrag_count,
            is_reliable: reliability,
            message_factory,
            unicast_locator_list,
            multicast_locator_list,
            config,
        }
    }
}

#[derive()]
pub struct Reader {
    guid: Guid,
    matched_writers: HashMap<Guid, WriterProxy>,
    cache: ReaderHistoryCache,
    receiver: MessageReceiver,
    htb_count: Count,
    htbfrag_count: Count,
    is_reliable: ReliabilityKind,
    message_factory: MessageFactory,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
    config: ReaderConfiguration,
}

impl Reader {
    pub fn get_guid(&self) -> Guid {
        self.guid
    }

    #[instrument(level = Level::ERROR, skip_all, fields(guid = %self.guid))]
    pub fn get_first_available_change(&self) -> Option<&CacheChangeContainer> {
        let mut available_changes = self.iter_all_available_changes(SampleStateKind::NotRead);
        available_changes.next()
    }

    #[instrument(level = Level::ERROR, skip_all, fields(guid = %self.guid))]
    pub fn take_first_available_change(&mut self) -> Option<CacheChangeContainer> {
        let (guid, sequence) = self
            .iter_all_available_changes(SampleStateKind::NotRead)
            .next()
            .map(|container| (container.get_guid(), container.get_sequence_number()))?;

        let Some(proxy) = self.matched_writers.get_mut(&guid) else {
            unreachable!()
        };
        proxy.clean(&sequence);

        self.cache.take_change(guid, sequence)
    }

    #[instrument(level = Level::ERROR, skip_all, fields(guid = %self.guid))]
    pub fn get_all_available_changes(
        &self,
        sample_state: SampleStateKind,
    ) -> Vec<&CacheChangeContainer> {
        self.iter_all_available_changes(sample_state).collect()
    }

    #[instrument(level = Level::ERROR, skip_all, fields(guid = %self.guid))]
    pub fn take_all_available_changes(&mut self) -> Vec<CacheChangeContainer> {
        let available_changes = self
            .iter_all_available_changes(SampleStateKind::NotRead)
            .map(|container| (container.get_guid(), container.get_sequence_number()))
            .collect::<Vec<_>>();
        let changes = available_changes
            .into_iter()
            .filter_map(|(guid, sequence)| self.cache.take_change(guid, sequence))
            .collect::<Vec<CacheChangeContainer>>();

        for change in &changes {
            let Some(proxy) = self.matched_writers.get_mut(&change.get_guid()) else {
                unreachable!()
            };
            proxy.clean(&change.get_sequence_number());
        }

        changes
    }

    #[instrument(level = Level::ERROR, skip_all, fields(guid = %self.guid))]
    pub fn ingest(&mut self, effects: &mut Effects, message: Message) -> Result<(), Error> {
        if self.matched_writers.is_empty() {
            return Ok(());
        };

        self.receiver.reset();
        self.receiver.capture_header(&message);

        if self.is_myself(&self.receiver) {
            let err_msg = "message is comming from myself".to_string();
            event!(Level::DEBUG, "{}", &err_msg);
            return Err(Error::FilteredOut { because: err_msg });
        }

        for submessage in message.submessages {
            self.receiver.capture_submessage_infos(&submessage);

            match submessage.content {
                SubmessageContent::Data {
                    extra_flags: _,
                    octets_to_inline_qos: _,
                    reader_id,
                    writer_id,
                    writer_sn,
                    inline_qos,
                    serialized_data,
                } => {
                    if !self.is_the_destination(reader_id) {
                        event!(Level::DEBUG, "submessage is not for me");
                        continue;
                    }

                    let writer_guid =
                        if matches!(writer_id, ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER) {
                            Guid::default()
                        } else {
                            Guid::new(self.receiver.source_guid_prefix, writer_id)
                        };

                    self.handle_data(
                        effects,
                        writer_guid,
                        submessage.header.flags,
                        writer_sn,
                        inline_qos.map(InlineQos::from),
                        serialized_data,
                    )?
                }
                SubmessageContent::Gap {
                    reader_id,
                    writer_id,
                    gap_start,
                    gap_list,
                    gap_group_info,
                    filtered_count,
                } => {
                    if !self.is_the_destination(reader_id) {
                        event!(Level::DEBUG, "submessage is not for me");
                        continue;
                    }

                    let writer_guid =
                        if matches!(writer_id, ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER) {
                            Guid::default()
                        } else {
                            Guid::new(self.receiver.source_guid_prefix, writer_id)
                        };

                    self.handle_gap(
                        writer_guid,
                        gap_start,
                        gap_list,
                        gap_group_info,
                        filtered_count,
                    )?;
                }
                SubmessageContent::Heartbeat {
                    reader_id,
                    writer_id,
                    first_sn,
                    last_sn,
                    count,
                    heartbeat_group_info,
                } => {
                    if !self.is_the_destination(reader_id) {
                        event!(Level::DEBUG, "submessage is not for me");
                        continue;
                    }

                    if count <= self.htb_count {
                        event!(Level::DEBUG, "HEARTBEAT discarded: counter wrong");
                        continue;
                    }
                    self.htb_count = count;

                    let writer_guid = Guid::new(self.receiver.source_guid_prefix, writer_id);

                    self.handle_heartbeat(
                        effects,
                        writer_guid,
                        first_sn,
                        last_sn,
                        heartbeat_group_info,
                    )?
                }
                SubmessageContent::DataFrag {
                    flags: _,
                    extra_flags: _,
                    octets_to_inline_qos: _,
                    reader_id,
                    writer_id,
                    writer_sn,
                    fragment_starting_num,
                    fragment_in_submessage,
                    fragment_size,
                    sample_size,
                    inline_qos,
                    serialized_payload,
                } => {
                    if !self.is_the_destination(reader_id) {
                        event!(Level::DEBUG, "submessage is not for me");
                        continue;
                    }

                    let writer_guid = Guid::new(self.receiver.source_guid_prefix, writer_id);
                    self.handle_datafrag(
                        effects,
                        writer_guid,
                        writer_sn,
                        fragment_starting_num,
                        fragment_in_submessage,
                        fragment_size,
                        sample_size,
                        inline_qos.map(InlineQos::from),
                        serialized_payload,
                    )?
                }
                SubmessageContent::HeartbeatFrag {
                    reader_id,
                    writer_id,
                    writer_sn,
                    last_fragmentation_num,
                    count,
                } => {
                    if !self.is_the_destination(reader_id) {
                        event!(Level::DEBUG, "submessage is not for me");
                        continue;
                    }

                    if count <= self.htbfrag_count {
                        event!(Level::DEBUG, "HEARTBEATFRAG discarded: counter wrong");
                        continue;
                    }
                    self.htbfrag_count = count;

                    let writer_guid = Guid::new(self.receiver.source_guid_prefix, writer_id);

                    self.handle_heartbeatfrag(
                        effects,
                        writer_guid,
                        writer_sn,
                        last_fragmentation_num,
                    )?
                }
                SubmessageContent::InfoTimestamp { timestamp } => {
                    self.receiver.rtps_send_timestamp = timestamp
                }
                SubmessageContent::AckNack { .. } => {
                    event!(Level::DEBUG, "Not supposed to receive a AckNack");
                    continue;
                }
                SubmessageContent::NackFrag { .. } => {
                    event!(Level::DEBUG, "Not supposed to receive a NackFrag");
                    continue;
                }
                SubmessageContent::InfoReply { .. } => {
                    event!(
                        Level::DEBUG,
                        "Received a InfoReply, i currently don't know what to do about it"
                    );
                    continue;
                }
                SubmessageContent::InfoDestination { .. } => {
                    event!(
                        Level::DEBUG,
                        "Received a InfoDestination, i currently don't know what to do about it"
                    );
                    continue;
                }
                SubmessageContent::InfoSource { .. } => {
                    event!(
                        Level::DEBUG,
                        "Received a InfoSource, i currently don't know what to do about it"
                    );
                    continue;
                }
                SubmessageContent::HeaderExtension { .. } => {
                    event!(
                        Level::DEBUG,
                        "Received a HeaderExtension, i currently don't know what to do about it"
                    );
                    continue;
                }
                SubmessageContent::Pad => {
                    event!(
                        Level::DEBUG,
                        "Received a Gap, i currently don't know what to do about it"
                    );
                    continue;
                }
                // FIXME: should not be needed, because Deserialization stage should discard any message not in correct format
                SubmessageContent::Unknown { data: _ } => {
                    event!(Level::DEBUG, "Unknown message received");
                    continue;
                }
            }
            self.cleanup();
        }

        Ok(())
    }

    #[instrument(level = Level::ERROR, skip_all, fields(guid = %self.guid))]
    pub fn tick(&mut self, effects: &mut Effects) {
        // TODO: check time related QoS

        for (_guid, proxy) in self.matched_writers.iter_mut() {
            let base = proxy.available_changes_max() + 1;
            let set: &Vec<SequenceNumber> = &proxy.missing_changes();
            let missing_set = SequenceNumberSet::new(base, set);

            let count = proxy.acknack_counter.increase();

            let mut msg = self
                .message_factory
                .message()
                .reader(self.guid.get_entity_id())
                .writer(proxy.remote_writer_guid.get_entity_id())
                .acknack(missing_set, count);

            let missing_changes_seq = proxy.missing_changes();

            for missing_change_seq in missing_changes_seq {
                let missing_set = if let Some(change) = self
                    .cache
                    .get_fragmented_change(proxy.get_remote_writer_guid(), missing_change_seq)
                {
                    let missing_frags = change.get_missing_fragments_number();

                    if missing_frags.is_empty() {
                        continue;
                    }

                    let missing_frags = missing_frags
                        .into_iter()
                        .map(|f| FragmentNumber(f.0 + 1))
                        .collect::<Vec<_>>();
                    let base = missing_frags.first().unwrap();
                    FragmentNumberSet::new(*base, &missing_frags)
                } else {
                    let Some(last_missing_frag) = proxy.last_announced_frag(missing_change_seq)
                    else {
                        continue;
                    };

                    let missing_frags = (0..=last_missing_frag.0)
                        .map(|f| FragmentNumber(f + 1))
                        .collect::<Vec<_>>();
                    FragmentNumberSet::new(FragmentNumber(1), &missing_frags)
                };

                let count = proxy.nackfrag_counter.increase();

                msg = msg.nackfrag(missing_change_seq, missing_set, count);
            }

            let message = msg.build();

            let effect = Effect::Message {
                timestamp_millis: Default::default(),
                message,
                locators: proxy.get_locators(),
            };

            effects.push(effect);
        }
    }

    pub fn add_proxy(&mut self, proxy: WriterProxy) {
        self.matched_writers
            .insert(proxy.get_remote_writer_guid(), proxy);
    }

    pub fn remove_proxy(&mut self, proxy_guid: Guid) {
        self.matched_writers.remove(&proxy_guid);
    }

    pub fn lookup_proxy(&mut self, proxy_guid: Guid) -> bool {
        self.matched_writers.contains_key(&proxy_guid)
    }

    pub fn add_unicast_locators(&mut self, mut locators: LocatorList) {
        self.unicast_locator_list.append(&mut locators);
    }

    pub fn add_multicast_locators(&mut self, mut locators: LocatorList) {
        self.multicast_locator_list.append(&mut locators);
    }

    fn has_unreaded_available_change(&self) -> bool {
        self.iter_all_available_changes(SampleStateKind::NotRead)
            .count()
            != 0
    }

    fn iter_all_available_changes(
        &self,
        sample_state: SampleStateKind,
    ) -> impl Iterator<Item = &CacheChangeContainer> {
        let available_changes_max_by_proxy: HashMap<Guid, SequenceNumber> = self
            .matched_writers
            .values()
            .map(|proxy| (proxy.remote_writer_guid, proxy.available_changes_max()))
            .collect();

        self.cache
            .get_changes()
            .filter(|container| container.get_sample_state_kind() == sample_state)
            .filter(|container| {
                if let Some(max_available_change) =
                    available_changes_max_by_proxy.get(&container.get_guid())
                {
                    container.get_sequence_number() <= *max_available_change
                } else {
                    false
                }
            })
            .sorted_by(|first, second| {
                Ord::cmp(
                    &(first.get_sequence_number(), first.get_reception_timestamp()),
                    &(
                        second.get_sequence_number(),
                        second.get_reception_timestamp(),
                    ),
                )
            })
    }

    fn cleanup(&mut self) {
        while let Some(change) = self.cache.collect_garbage() {
            let Some(proxy) = self.matched_writers.get_mut(&change.get_guid()) else {
                unreachable!()
            };
            proxy.clean(&change.get_sequence_number());
        }
    }

    fn is_myself(&self, receiver: &MessageReceiver) -> bool {
        let self_guid_prefix = self.guid.get_guid_prefix();
        let remote_guid_prefix = receiver.source_guid_prefix;
        self_guid_prefix == remote_guid_prefix
    }

    fn is_the_destination(&self, entity_id: EntityId) -> bool {
        let self_entity_id = self.guid.get_entity_id();
        self_entity_id == entity_id || entity_id == ENTITYID_UNKOWN
    }

    fn handle_data(
        &mut self,
        effects: &mut Effects,
        writer_guid: Guid,
        flags: SubmessageFlags,
        change_sequence: SequenceNumber,
        inline_qos: Option<InlineQos>,
        data: Option<SerializedData>,
    ) -> Result<(), Error> {
        let Some(proxy) = self.matched_writers.get_mut(&writer_guid) else {
            return Err(Error::RemoteWriterNotFound(writer_guid));
        };

        let payload_nature = if flags.third() == 1 && flags.fourth() == 0 {
            ContentNature::Data
        } else if flags.third() == 0 && flags.fourth() == 1 {
            ContentNature::Key
        } else {
            ContentNature::None
        };

        // FIXME: temporary, should be smarter when interop will be needed
        if !matches!(payload_nature, ContentNature::Data) {
            return Ok(());
        }

        // FIXME: when interop will be needed, implement the three ways to get the key
        // key in qos
        // key in submessage
        // key calculated from deserialized message
        let mut instance_handle = Default::default();

        if let Some(qos) = inline_qos.as_ref() {
            instance_handle = qos.key_hash;
        }

        let mut change = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            change_sequence,
            // FIXME: fragment_size should be retrieved from configuration
            60 * 1024,
            self.receiver.rtps_reception_timestamp,
            data,
        );

        if let Some(qos) = inline_qos {
            change.set_qos(qos);
        }

        if let Some(emission_timestamp) = self.receiver.rtps_send_timestamp {
            change.set_emission_timestamp(emission_timestamp);
        }

        if let Err(e) = self.cache.push_change(change) {
            // TODO: trace error
            // TODO: for now there are no error that could possibly be raised
            unreachable!()
        };

        let is_in_watch_all_mode = proxy.get_remote_writer_guid() == Guid::default();

        let expected_sequence = proxy.expected_sequence();

        let is_the_expected_sequence = change_sequence < expected_sequence;
        let is_change_already_received = proxy.is_change_received(change_sequence);

        if !is_in_watch_all_mode && (is_the_expected_sequence || is_change_already_received) {
            event!(
                Level::DEBUG,
                is_in_watch_all_mode,
                is_the_expected_sequence,
                is_change_already_received,
                "DATA - Pre-conditions not met"
            );
            return Ok(());
        }

        if matches!(self.is_reliable, ReliabilityKind::BestEffort)
            && change_sequence > expected_sequence
        {
            proxy.lost_changes_update(change_sequence, false);
        }

        proxy.received_change_set(change_sequence);

        if self.has_unreaded_available_change() {
            effects.push(Effect::DataAvailable);
        }

        event!(Level::DEBUG, "DATA processed");

        Ok(())
    }

    fn handle_gap(
        &mut self,
        writer_guid: Guid,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
        _gap_group_info: Option<GapGroupInfo>,
        _filtered_count: Option<ChangeCount>,
    ) -> Result<(), Error> {
        let Some(proxy) = self.matched_writers.get_mut(&writer_guid) else {
            return Err(Error::RemoteWriterNotFound(writer_guid));
        };

        if !Self::is_gap_valid(&gap_start, &gap_list) {
            event!(Level::DEBUG, "GAP invalid");
            return Err(Error::InvalidGapError);
        }

        let expected_sequence = proxy.expected_sequence();

        if matches!(self.is_reliable, ReliabilityKind::Reliable) || gap_start >= expected_sequence {
            let mut list = (gap_start.0..gap_list.get_base().0)
                .map(SequenceNumber)
                .collect::<Vec<_>>();
            let set = gap_list.get_set();
            list.extend(set);

            proxy.not_available_change_set(list, ChangeCount::default());

            event!(Level::DEBUG, "GAP processed");
        } else {
            event!(Level::DEBUG, "GAP discarded");
        }

        Ok(())
    }

    fn is_gap_valid(gap_start: &SequenceNumber, gap_list: &SequenceNumberSet) -> bool {
        gap_start.0 <= 0 || gap_start > &gap_list.get_base()
    }

    fn handle_heartbeat(
        &mut self,
        effects: &mut Effects,
        writer_guid: Guid,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        _heartbeat_group_info: Option<HeartbeatGroupInfo>,
    ) -> Result<(), Error> {
        let Some(proxy) = self.matched_writers.get_mut(&writer_guid) else {
            return Err(Error::RemoteWriterNotFound(writer_guid));
        };

        let now = Utc::now().timestamp_millis();

        let time_diff = now.saturating_sub(proxy.last_heartbeat_timestamp_ms);

        if time_diff > self.config.heartbeat_suppression_delay_ms {
            event!(
                Level::DEBUG,
                "Suppression delay preventing processing this Heartbeat"
            );
            return Ok(());
        }
        proxy.last_heartbeat_timestamp_ms = now;

        proxy.missing_changes_update(last_sn);
        proxy.lost_changes_update(first_sn, true);
        proxy.last_missing_frag_remove_until(first_sn);

        let is_final = self.receiver.flags.second() == 0;
        let missings = proxy.missing_changes();
        let are_missings = !missings.is_empty();

        if is_final && are_missings {
            let effect = Effect::ScheduleTick {
                delay: self.config.heartbeat_response_delay_ms,
            };
            effects.push(effect);
        }

        event!(Level::DEBUG, "HEARTBEAT processed");

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_datafrag(
        &mut self,
        effects: &mut Effects,
        writer_guid: Guid,
        sequence: SequenceNumber,
        fragment_starting_num: FragmentNumber,
        fragment_in_submessage: u16,
        fragment_size: u16,
        sample_size: u32,
        inline_qos: Option<InlineQos>,
        data: SerializedData,
    ) -> Result<(), Error> {
        let Some(proxy) = self.matched_writers.get_mut(&writer_guid) else {
            return Err(Error::RemoteWriterNotFound(writer_guid));
        };

        let expected_sequence = proxy.expected_sequence();

        let is_expected_sequence = sequence < expected_sequence;
        let is_change_already_received = proxy.is_change_received(sequence);

        if is_expected_sequence || is_change_already_received {
            event!(Level::DEBUG, "Pre-conditions not met");
            return Ok(());
        }

        if matches!(self.is_reliable, ReliabilityKind::BestEffort) && sequence > expected_sequence {
            event!(
                Level::DEBUG,
                "This sequence is beyond the one that was expected"
            );
            proxy.lost_changes_update(sequence, false);
            // for lost_change in proxy.lost_changes() {
            //     let _res = self
            //         .cache
            //         .remove_one(lost_change, proxy.get_remote_writer_guid());
            // }
        }

        // TODO: check if these fragments are already received

        match self.cache.get_fragmented_change_mut(writer_guid, sequence) {
            None => {
                // FIXME: when interop will be needed, implement the three ways to get the key
                // key in qos
                // key in submessage
                // key calculated from deserialized message
                let mut instance_handle = Default::default();

                if let Some(qos) = inline_qos.as_ref() {
                    instance_handle = qos.key_hash;
                }

                let mut change = FragmentedCacheChange::new(
                    ChangeKind::Alive,
                    writer_guid,
                    instance_handle,
                    sequence,
                    fragment_size,
                    self.receiver.rtps_reception_timestamp,
                    sample_size,
                );

                if let Some(qos) = inline_qos {
                    change.set_qos(qos);
                }

                if let Some(emission_timestamp) = self.receiver.rtps_send_timestamp {
                    change.set_emission_timestamp(emission_timestamp);
                }

                self.cache.push_fragmented_change(change);

                Ok(())
            }
            Some(frag_change) => {
                if let Err(_e) = frag_change.check_validity(fragment_size, sample_size) {
                    event!(
                        Level::ERROR,
                        "The fragments caracteristics does not match the one already known for this change"
                    );
                    // TODO: return with an error
                    todo!()
                }

                if !frag_change.are_fragments_missing(fragment_starting_num, fragment_in_submessage)
                {
                    event!(Level::DEBUG, "The fragments has already been processed");
                    return Ok(());
                }

                frag_change.insert_fragments(
                    fragment_starting_num,
                    fragment_in_submessage,
                    fragment_size,
                    &data,
                );

                if frag_change.is_complete() {
                    match self.cache.transfer(writer_guid, sequence) {
                        Ok(()) => {
                            if self.has_unreaded_available_change() {
                                effects.push(Effect::DataAvailable);
                            }
                            Ok(())
                        }
                        Err(_e) => {
                            // TODO: for now there are no error than can be raised here
                            unreachable!()
                        }
                    }
                } else {
                    Ok(())
                }
            }
        }
    }

    fn handle_heartbeatfrag(
        &mut self,
        effects: &mut Effects,
        writer_guid: Guid,
        sequence: SequenceNumber,
        last_fragmentation_num: FragmentNumber,
    ) -> Result<(), Error> {
        let Some(proxy) = self.matched_writers.get_mut(&writer_guid) else {
            return Err(Error::RemoteWriterNotFound(writer_guid));
        };

        let now = Utc::now().timestamp_millis();

        let time_diff = now.saturating_sub(proxy.last_heartbeat_timestamp_ms);

        if time_diff > self.config.heartbeat_suppression_delay_ms {
            event!(
                Level::DEBUG,
                "Suppression delay preventing processing this Heartbeat"
            );
            return Ok(());
        }
        proxy.last_heartbeat_timestamp_ms = now;

        proxy.missing_changes_update(sequence);
        proxy.set_last_announced_frag(sequence, FragmentNumber(last_fragmentation_num.0 - 1));

        let missings = proxy.missing_changes();
        let are_missings = !missings.is_empty();

        if are_missings {
            let effect = Effect::ScheduleTick {
                delay: self.config.heartbeat_response_delay_ms,
            };
            effects.push(effect);
        }

        Ok(())
    }

    pub fn extract_proxy(&self) -> ReaderProxy {
        ReaderProxy {
            remote_reader_guid: self.get_guid(),
            expects_inline_qos: false,
            is_active: true,
            unicast_locator_list: self.unicast_locator_list.clone(),
            multicast_locator_list: self.multicast_locator_list.clone(),
            ..Default::default()
        }
    }
}

impl Debug for Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reader")
            .field("guid", &self.guid)
            .field("matched_writers", &self.matched_writers)
            .field("cache", &self.cache)
            .field("receiver", &self.receiver)
            .field("htb_count", &self.htb_count)
            .field("htbfrag_count", &self.htbfrag_count)
            .field("is_reliable", &self.is_reliable)
            .field("configuration", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::MessageFactory,
        types::{
            ContentNature, EntityId, Guid, InlineQos, LocatorList, ReliabilityKind, SequenceNumber,
            SerializedData,
        },
    };
    use rstest::{fixture, rstest};

    use crate::{
        Effects, WriterProxy,
        common::{
            Effect,
            tests::{setup_reader_0_guid, setup_uni_locatorlist, setup_writer_0_guid},
        },
    };

    use super::{Reader, ReaderBuilder};

    #[rstest]
    fn ingest_test(
        #[from(setup_reader)] mut reader: Reader,
        #[from(setup_reader_0_guid)] reader_guid_0: Guid,
        #[from(setup_writer_0_guid)] writer_guid_0: Guid,
    ) {
        let mut effects = Effects::new();

        let message = MessageFactory::new(writer_guid_0.get_guid_prefix())
            .message()
            .reader(reader_guid_0.get_entity_id())
            .writer(writer_guid_0.get_entity_id())
            .data(
                ContentNature::Data,
                SequenceNumber(1),
                None,
                Some(SerializedData::from_vec(vec![0, 0, 0, 0])),
            )
            .build();

        reader.ingest(&mut effects, message).unwrap();
        let result = effects.pop().unwrap();

        assert!(matches!(result, Effect::DataAvailable));

        let change = reader.get_first_available_change().unwrap();

        assert_eq!(change.get_sequence_number(), SequenceNumber(1));
    }

    #[fixture]
    fn setup_reader(
        #[default(ReliabilityKind::BestEffort)] reliable: ReliabilityKind,
        #[default(InlineQos::default())] qos: InlineQos,
        #[from(setup_writer_proxy)] proxy: WriterProxy,
    ) -> Reader {
        let mut reader = ReaderBuilder::new(Guid::default(), qos)
            .reliability(reliable)
            .build();
        reader.add_proxy(proxy);
        reader
    }

    #[fixture]
    fn setup_writer_proxy(
        #[from(setup_writer_0_guid)] guid: Guid,
        #[from(setup_uni_locatorlist)] uni_locators: LocatorList,
    ) -> WriterProxy {
        WriterProxy::new(
            guid,
            EntityId::default(),
            60 * 1024,
            uni_locators,
            LocatorList::default(),
        )
    }
}
