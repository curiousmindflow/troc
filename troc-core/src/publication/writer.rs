use std::collections::HashMap;

use crate::{
    messages::{Message, MessageFactory, MessageReceiver, SubmessageContent},
    types::{
        ChangeKind, ContentNature, ENTITYID_UNKOWN, EntityId, FragmentNumber, Guid,
        HistoryQosPolicy, InlineQos, InstanceHandle, LocatorList, ReliabilityKind, SequenceNumber,
        SequenceNumberSet, SerializedData, Timestamp, sequence_number::SEQUENCENUMBER_UNKNOWN,
    },
};
use chrono::Utc;
use tracing::{Level, event};

use crate::{
    CacheChange, WriterProxy,
    common::{CacheChangeInfos, Counter, Effect, Effects, Error, ReaderProxy},
    publication::{WriterHistoryCache, historycache::WriterHistoryCacheConfiguration},
};

#[derive(Debug)]
pub struct WriterConfiguration {
    push_mode: bool,
    fragment_size: u16,
    heartbeat_period: u64,
    nack_response_delay_ms: i64,
    nack_suppression_delay_ms: i64,
    should_piggyback_heartbeat: bool,
    should_piggyback_timestamp: bool,
}

impl Default for WriterConfiguration {
    fn default() -> Self {
        Self {
            push_mode: true,
            fragment_size: 60 * 1024,
            heartbeat_period: 5000,
            nack_response_delay_ms: 200,
            nack_suppression_delay_ms: 0,
            should_piggyback_heartbeat: true,
            should_piggyback_timestamp: true,
        }
    }
}

#[derive(Default)]
pub struct WriterBuilder {
    guid: Guid,
    qos: InlineQos,
    reliability: ReliabilityKind,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
    config: WriterConfiguration,
}

impl WriterBuilder {
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

    pub fn with_configuration(mut self, config: WriterConfiguration) -> Self {
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

    pub fn build(self) -> Writer {
        let WriterBuilder {
            guid,
            qos,
            reliability: is_reliable,
            unicast_locator_list,
            multicast_locator_list,
            config,
        } = self;
        let matched_readers = Default::default();
        let depth = match qos.history {
            HistoryQosPolicy::KeepAll => None,
            HistoryQosPolicy::KeepLast { depth } => Some(depth),
        };
        let cache_config = WriterHistoryCacheConfiguration::new(depth, 100, 100, 100);
        let cache = WriterHistoryCache::new(cache_config);
        Writer {
            guid,
            qos,
            is_reliable,
            message_factory: MessageFactory::new(guid.get_guid_prefix()),
            matched_readers,
            cache,
            receiver: MessageReceiver::new(guid.get_guid_prefix()),
            last_change_sequence_number: SEQUENCENUMBER_UNKNOWN,
            heartbeat_counter: Counter::default(),
            heartbeatfrag_counter: Counter::default(),
            unicast_locator_list,
            multicast_locator_list,
            config,
        }
    }
}

#[derive(Debug)]
pub struct Writer {
    guid: Guid,
    qos: InlineQos,
    is_reliable: ReliabilityKind,
    message_factory: MessageFactory,
    matched_readers: HashMap<Guid, ReaderProxy>,
    cache: WriterHistoryCache,
    receiver: MessageReceiver,
    last_change_sequence_number: SequenceNumber,
    heartbeat_counter: Counter,
    heartbeatfrag_counter: Counter,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
    config: WriterConfiguration,
}

impl Writer {
    pub fn get_guid(&self) -> Guid {
        self.guid
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<SerializedData>,
        inline_qos: Option<InlineQos>,
        instance_handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number += 1;
        let sample_size = data.as_ref().map(|d| d.size()).unwrap_or_default() as u32;
        let fragments_count = sample_size.div_ceil(self.config.fragment_size as u32) as u16;

        CacheChange {
            infos: CacheChangeInfos {
                kind,
                writer_guid: self.guid,
                instance_handle,
                sequence_number: self.last_change_sequence_number,
                sample_size,
                fragment_size: self.config.fragment_size,
                fragments_count,
                emission_timestamp: Some(Timestamp::from_datetime(Utc::now())),
                reception_timestamp: Timestamp::from_datetime(Utc::now()),
                inline_qos,
            },
            data,
        }
    }

    pub fn add_change(&mut self, effects: &mut Effects, change: CacheChange) -> Result<(), Error> {
        self.cache.push_change(change).unwrap();
        self.produce_data(self.last_change_sequence_number, effects)
    }

    /// Process an incomming message if it contains a Acknack or a NackFrag submessage
    ///
    /// Each Acknack will records positive and/or negative acknowledgments and call [`NackedDataSchedulePort::on_nacked_data_schedule`] if one was provided
    pub fn ingest(&mut self, effects: &mut Effects, message: Message) -> Result<(), Error> {
        if matches!(self.is_reliable, ReliabilityKind::BestEffort) {
            // FIXME: maybe this check can be done at compile time through type system wizardry
            return Err(Error::IsBestEffort);
        }

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
                SubmessageContent::AckNack {
                    reader_id,
                    writer_id,
                    writer_sn_state,
                    count,
                } => {
                    if !self.is_the_destination(writer_id) {
                        event!(Level::DEBUG, "submessage is not for me");
                        continue;
                    }

                    let reader_guid = Guid::new(self.receiver.source_guid_prefix, reader_id);

                    let Some(proxy) = self.matched_readers.get_mut(&reader_guid) else {
                        event!(Level::DEBUG, "ReaderProxy is unknown");
                        continue;
                    };

                    if count <= proxy.acknack_count {
                        event!(Level::DEBUG, "HEARTBEAT discarded: counter wrong");
                        continue;
                    }
                    proxy.acknack_count = count;

                    let base = writer_sn_state.get_base();
                    let set = writer_sn_state.get_set();

                    proxy.acked_changes_set(base - 1);
                    proxy.requested_changes_set(set);

                    let effect = Effect::ScheduleTick {
                        delay: self.config.nack_response_delay_ms,
                    };
                    effects.push(effect);
                }
                SubmessageContent::NackFrag {
                    reader_id,
                    writer_id,
                    writer_sn,
                    fragmentation_number_state,
                    count,
                } => {
                    if !self.is_the_destination(writer_id) {
                        event!(Level::DEBUG, "submessage is not for me");
                        continue;
                    }

                    let Some(proxy) = self
                        .matched_readers
                        .get_mut(&Guid::new(self.receiver.source_guid_prefix, reader_id))
                    else {
                        event!(Level::DEBUG, "ReaderProxy is unknown");
                        continue;
                    };

                    if count <= proxy.nackfrag_count {
                        event!(Level::DEBUG, "HEARTBEAT discarded: counter wrong");
                        continue;
                    }
                    proxy.nackfrag_count = count;

                    todo!()
                }
                _ => {
                    event!(Level::DEBUG, "Unexpected submessage received");
                }
            }
        }

        Ok(())
    }

    pub fn tick(&mut self, effects: &mut Effects) {
        if matches!(self.is_reliable, ReliabilityKind::BestEffort) {
            return;
        }

        let max_seq_in_cache = self
            .cache
            .get_max_sequence()
            .unwrap_or(SEQUENCENUMBER_UNKNOWN);

        for proxy in self.matched_readers.values_mut() {
            // HEARTBEAT production
            if proxy.unacked_changes(max_seq_in_cache) {
                let max = self.last_change_sequence_number;
                let min = self.cache.get_min_sequence().unwrap_or(max + 1);

                let count = self.heartbeat_counter.increase();

                let mut msg = self
                    .message_factory
                    .message()
                    .reader(proxy.get_remote_reader_guid().get_entity_id())
                    .writer(self.guid.get_entity_id())
                    .heartbeat(false, false, min, max, count, None);

                let last_frag_per_sequence = self.cache.get_last_frag_per_sequence();

                // this check if the number of heartbeat submessages will not making the Message length greater than the maximum UDP packet
                assert!(last_frag_per_sequence.len() < 2000);

                for (sequence, last_frag_num) in last_frag_per_sequence {
                    let count = self.heartbeatfrag_counter.increase();
                    msg = msg.heartbeatfrag(sequence, FragmentNumber(last_frag_num.0 + 1), count)
                }

                let message = msg.build();

                let effect = Effect::Message {
                    timestamp_millis: Default::default(),
                    message,
                    locators: proxy.get_locators(),
                };
                effects.push(effect);
            } else {
                event!(Level::DEBUG, "No unacked changes");
            }

            // NACKED production
            if proxy.are_requested_changes() {
                let mut msg = self
                    .message_factory
                    .message()
                    .reader(proxy.get_remote_reader_guid().get_entity_id())
                    .writer(self.guid.get_entity_id());

                let mut gaps = Vec::new();
                while let Some(request_change_sequence) = proxy.next_requested_change() {
                    if let Some(requested_change) = self.cache.get_change(request_change_sequence) {
                        msg = msg.data(
                            ContentNature::Data,
                            request_change_sequence,
                            requested_change.get_inline_qos().cloned(),
                            requested_change.data.clone(),
                        );
                        // TODO: flag 'requested_change_sequence' as not a requested change anymore (not acked, neither) for if there is
                        // a call to this method (Self::produce_nacked_data(..) several times consecutively, this message will not be produced each time)
                    } else {
                        gaps.push(request_change_sequence);
                    }
                }

                if let Some((gap_first, gap_list)) = Self::build_gap_infos(&gaps) {
                    msg = msg.gap(gap_first, gap_list, None, None);
                }

                let message = msg.build();

                let effect = Effect::Message {
                    timestamp_millis: Default::default(),
                    message,
                    locators: proxy.get_locators(),
                };
                effects.push(effect);
            } else {
                event!(Level::DEBUG, "No requested changes");
            }
        }
    }

    pub fn add_proxy(&mut self, proxy: ReaderProxy) {
        self.matched_readers
            .insert(proxy.get_remote_reader_guid(), proxy);
    }

    pub fn remove_proxy(&mut self, proxy_guid: Guid) {
        self.matched_readers.remove(&proxy_guid);
    }

    pub fn lookup_proxy(&mut self, proxy_guid: Guid) -> bool {
        self.matched_readers.contains_key(&proxy_guid)
    }

    pub fn add_unicast_locators(&mut self, mut locators: LocatorList) {
        self.unicast_locator_list.append(&mut locators);
    }

    pub fn add_multicast_locators(&mut self, mut locators: LocatorList) {
        self.multicast_locator_list.append(&mut locators);
    }

    fn produce_data(
        &mut self,
        sequence: SequenceNumber,
        effects: &mut Effects,
    ) -> Result<(), Error> {
        if self.matched_readers.is_empty() {
            return Ok(());
        }

        let Some(change) = self.cache.get_change(sequence) else {
            return Ok(());
        };

        let mut msg = self
            .message_factory
            .message()
            .reader(ENTITYID_UNKOWN)
            .writer(self.guid.get_entity_id());

        if self.config.should_piggyback_timestamp {
            msg = msg.info_timestamp(Some(change.get_emission_timestamp().unwrap()));
        }

        msg = msg.data(
            ContentNature::Data,
            change.get_sequence_number(),
            change.get_inline_qos().cloned(),
            change.get_data().cloned(),
        );

        if matches!(self.is_reliable, ReliabilityKind::Reliable)
            && self.config.should_piggyback_heartbeat
        {
            let count = self.heartbeat_counter.increase();
            let max = self.last_change_sequence_number;
            let min = self.cache.get_min_sequence().unwrap_or(max + 1);
            msg = msg.heartbeat(true, false, min, max, count, None);
        }

        let message = msg.build();

        let locators = self
            .matched_readers
            .values()
            .filter(|p| p.can_send())
            .flat_map(|p| p.get_locators().get_inner())
            .collect::<Vec<_>>();
        let locators = LocatorList::from(locators.as_slice());

        let effect = Effect::Message {
            timestamp_millis: Default::default(),
            message,
            locators,
        };
        effects.push(effect);

        self.matched_readers
            .values_mut()
            .for_each(|p| p.set_highest_sent_change_sn(sequence));

        Ok(())
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

    fn build_gap_infos(gaps: &[SequenceNumber]) -> Option<(SequenceNumber, SequenceNumberSet)> {
        let gap_first = *gaps.first()?;

        let mut gap_last = gap_first;
        let mut gap_last_pos = 0;
        let mut gaps_peekable_iter = gaps.iter().peekable();
        while let Some((pos, &gap)) = gaps_peekable_iter.clone().enumerate().next() {
            gap_last = gap;
            gap_last_pos = pos;
            let Some(&&next) = gaps_peekable_iter.peek() else {
                break;
            };
            if next > gap + 1 {
                break;
            }
        }
        let gap_unconsecutives = gaps
            .iter()
            .skip(gap_last_pos + 1)
            .cloned()
            .collect::<Vec<_>>();

        Some((
            gap_first,
            SequenceNumberSet::new(gap_last, &gap_unconsecutives),
        ))
    }

    pub fn extract_proxy(&self) -> WriterProxy {
        WriterProxy {
            remote_writer_guid: self.guid,
            unicast_locator_list: self.unicast_locator_list.clone(),
            multicast_locator_list: self.multicast_locator_list.clone(),
            data_max_size_serialized: 59 * 1024,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        messages::{Message, MessageFactory},
        types::{
            ChangeKind, Count, EntityId, Guid, InlineQos, InstanceHandle, LocatorList,
            ReliabilityKind, SequenceNumber, SequenceNumberSet, SerializedData,
        },
    };
    use rstest::{fixture, rstest};

    use crate::{
        CacheChange, Effects, ReaderProxy,
        common::{
            Effect, EffectConsumption,
            tests::{setup_reader_0_guid, setup_uni_locatorlist, setup_writer_0_guid},
        },
    };

    use super::{Writer, WriterBuilder, WriterConfiguration};

    #[rstest]
    fn new_change(#[from(setup_writer)] mut writer: Writer) {
        let initial_start_sequence_number = writer.last_change_sequence_number;

        let _change = new_change_helper(&mut writer);

        assert_eq!(
            writer.last_change_sequence_number,
            initial_start_sequence_number + 1
        );
    }

    #[rstest]
    fn add_change(#[from(setup_writer)] mut writer: Writer) {
        let mut effects = Effects::new();

        let change = new_change_helper(&mut writer);

        writer.add_change(&mut effects, change).unwrap();

        effects.consume(|e| match e {
            Effect::Message {
                timestamp_millis: _,
                message: _,
                locators: _,
            } => EffectConsumption::Consume,
            _ => EffectConsumption::Left,
        });

        assert!(effects.is_empty());
    }

    #[rstest]
    fn ingest(
        #[from(setup_writer)]
        #[with(ReliabilityKind::Reliable)]
        mut writer: Writer,
        #[from(setup_ack)] message: Message,
    ) {
        let mut effects = Effects::new();
        let change = writer.new_change(
            ChangeKind::Alive,
            Some(SerializedData::from_vec(vec![0, 1, 2, 3])),
            None,
            InstanceHandle::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        );
        writer.add_change(&mut effects, change).unwrap();
        effects.clean();

        writer.ingest(&mut effects, message).unwrap();

        effects.consume(|e| {
            let Effect::ScheduleTick { delay } = e else {
                panic!()
            };
            // TODO: maybe check delay value
            EffectConsumption::Consume
        });
        // TODO: check internal fields to assert good state
        // -> which fields to check ?
        let reader_proxy = writer.matched_readers.values().next().unwrap();
        assert_eq!(reader_proxy.highest_sent_change_sn, SequenceNumber(1));
        assert!(effects.is_empty());
    }

    #[rstest]
    fn tick(
        #[from(setup_writer)]
        #[with(ReliabilityKind::Reliable)]
        mut writer: Writer,
    ) {
        let mut effects = Effects::new();

        // TODO:
        let reader_proxy = writer.matched_readers.values_mut().next().unwrap();
        reader_proxy.highest_sent_change_sn = SequenceNumber(1);
        // tweak internal fields value to let Writer believe he has acknowledged changes / unacknowledged changes
        // call tick()
        writer.tick(&mut effects);
        // assert Message effect is produced, with correct content
        // TODO: effects should contains a Message (Heartbeat) and a Message (Data)
    }

    fn new_change_helper(writer: &mut Writer) -> CacheChange {
        writer.new_change(
            ChangeKind::Alive,
            Some(SerializedData::from_vec(vec![0, 1, 2, 3])),
            None,
            InstanceHandle::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        )
    }

    #[fixture]
    fn setup_ack(
        #[from(setup_reader_0_guid)] reader_guid_0: Guid,
        #[from(setup_writer_0_guid)] writer_guid_0: Guid,
    ) -> Message {
        MessageFactory::new(writer_guid_0.get_guid_prefix())
            .message()
            .reader(reader_guid_0.get_entity_id())
            .writer(writer_guid_0.get_entity_id())
            .acknack(
                SequenceNumberSet::new(SequenceNumber(1), &[SequenceNumber(0)]),
                Count::new(0),
            )
            .build()
    }

    #[fixture]
    fn setup_writer(
        #[default(ReliabilityKind::BestEffort)] reliable: ReliabilityKind,
        #[default(InlineQos::default())] qos: InlineQos,
        #[from(setup_reader_proxy)] proxy: ReaderProxy,
    ) -> Writer {
        let conf = WriterConfiguration {
            fragment_size: 60 * 1024,
            ..Default::default()
        };
        let mut writer = WriterBuilder::new(Guid::default(), qos)
            .with_configuration(conf)
            .reliability(reliable)
            .build();
        writer.add_proxy(proxy);
        writer
    }

    #[fixture]
    fn setup_reader_proxy(
        #[from(setup_writer_0_guid)] guid: Guid,
        #[from(setup_uni_locatorlist)] uni_locators: LocatorList,
    ) -> ReaderProxy {
        ReaderProxy::new(
            guid,
            EntityId::default(),
            false,
            true,
            uni_locators,
            LocatorList::default(),
        )
    }
}
