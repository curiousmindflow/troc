#[derive(Clone)]
pub struct Subscriber {}

impl Subscriber {
    pub(crate) fn new(
        guid: Guid,
        default_unicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        participant: Weak<Mutex<DomainParticipantInner>>,
        disc_cmd_sender: Sender<DiscoveryCommand>,
        reader_entity_identifier: Arc<Mutex<EntityIdentifier>>,
        qos: QosPolicy,
        wire_factory: WireFactory,
        config: Configuration,
        participant_span: Span,
    ) -> Self {
        // event!(Level::TRACE, "subscriber new");
        // let inner = Arc::new(Mutex::new(SubscriberInner::new(
        //     guid,
        //     participant,
        //     disc_cmd_sender.clone(),
        // )));
        // Self {
        //     guid,
        //     default_unicast_locator_list,
        //     default_multicast_locator_list,
        //     inner,
        //     disc_cmd_sender,
        //     reader_entity_identifier,
        //     qos,
        //     wire_factory,
        //     config,
        //     participant_span,
        //     span: Span::current(),
        // }

        todo!()
    }

    pub fn get_guid_prefix(&self) -> GuidPrefix {
        self.guid.get_guid_prefix()
    }

    pub fn get_entity_id(&self) -> EntityId {
        self.guid.get_entity_id()
    }

    pub async fn create_datareader<T>(
        &mut self,
        topic: &Topic,
        qos: &QosPolicy,
    ) -> Result<DataReader<T>, DdsError>
    where
        for<'a> T: Deserialize<'a> + Keyed + 'static,
    {
        // self.create_datareader_base(topic, qos, Default::default())
        //     .await

        todo!()
    }

    pub async fn create_datareader_with_params<T>(
        &mut self,
        topic: &Topic,
        qos: &QosPolicy,
        params: DataReaderParams,
    ) -> Result<DataReader<T>, DdsError>
    where
        for<'a> T: Deserialize<'a> + Keyed + 'static,
    {
        // self.create_datareader_base(topic, qos, params).await

        todo!()
    }

    async fn create_datareader_base<T>(
        &mut self,
        topic: &Topic,
        qos: &QosPolicy,
        params: DataReaderParams,
    ) -> Result<DataReader<T>, DdsError>
    where
        for<'a> T: Deserialize<'a> + Keyed + 'static,
    {
        // // TODO: merge qos and _topic.qos into qos

        // let reliability_level = match qos.reliability() {
        //     ReliabilityQosPolicy::BestEffort => ReliabilityKind::BestEffort,
        //     ReliabilityQosPolicy::Reliable { .. } => ReliabilityKind::Reliable,
        // };

        // let mut inline_qos: InlineQos = (*qos).into();
        // inline_qos.topic_name = topic.topic_name.clone();
        // inline_qos.type_name = topic.type_name.clone();

        // let DataReaderParams { name, listener } = params;

        // // TODO: implement compute_instance_handle
        // let rtps_reader = self
        //     .create_reader_base(
        //         topic.topic_kind,
        //         reliability_level,
        //         true,
        //         inline_qos,
        //         name,
        //         listener,
        //     )
        //     .await
        //     .map_err(|e| DdsError::Error(e.to_string()))?;
        // let data_reader = DataReader::new(rtps_reader, qos, self.span.clone()).await;
        // Ok(data_reader)

        todo!()
    }

    async fn create_reader_base(
        &mut self,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        expects_inline_qos: bool,
        qos: InlineQos,
        name: String,
        listener: bool,
    ) -> Result<Reader, DdsError> {
        // let reader_key = self.reader_entity_identifier.lock().await.get_new_key();
        // let reader_id = if matches!(topic_kind, TopicKind::WithKey) {
        //     EntityId::reader_with_key(reader_key.0)
        // } else {
        //     EntityId::reader_no_key(reader_key.0)
        // };
        // let reader_guid = Guid::new(self.guid.get_guid_prefix(), reader_id);

        // let to_one_wire_localhost = self
        //     .wire_factory
        //     .build_user_wire(Some("127.0.0.1".parse().unwrap()))
        //     .unwrap();
        // let to_one_wire_local_ip = self.wire_factory.build_user_wire(None).unwrap();
        // let to_one_wire_list = WireList::new(vec![to_one_wire_localhost, to_one_wire_local_ip]);
        // let to_many_wire_list = WireList::default();

        // let reader = Reader::new(
        //     reader_guid,
        //     name,
        //     topic_kind,
        //     reliability_level,
        //     Default::default(),
        //     to_one_wire_list.extract_locators(),
        //     to_many_wire_list.extract_locators(),
        //     expects_inline_qos,
        //     qos.clone(),
        //     listener,
        //     self.config.clone(),
        //     self.span.clone(),
        // )
        // .await;

        // reader
        //     .launch_reception_tasks(to_one_wire_list, to_many_wire_list)
        //     .await;

        // let reader_proxy = ReaderProxy::from_base_reader(&reader);

        // let old = self
        //     .inner
        //     .lock()
        //     .await
        //     .readers
        //     .insert(reader.get_guid().get_entity_id(), reader.duplicate());

        // assert!(old.is_none());

        // let data = DiscoveredReaderData {
        //     proxy: reader_proxy,
        //     params: qos,
        // };

        // event!(
        //     Level::TRACE,
        //     "Reader <{}> created, DiscoveredReaderData<{}> sent to discovery",
        //     reader_guid,
        //     data.proxy.remote_reader_guid
        // );

        // self.disc_cmd_sender
        //     .send(DiscoveryCommand::ReaderUpdate(data))
        //     .await
        //     .unwrap();

        // Ok(reader)

        todo!()
    }
}
