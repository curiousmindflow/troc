use std::sync::{Arc, Mutex};

use troc_key::Keyed;
use protocol::DdsError;
use protocol::types::{EntityId, Guid, GuidPrefix, LocatorList, TopicKind};
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::{
    discovery::EndpointLifecycleCommand,
    domain::{Configuration, EntityIdentifier},
    infrastructure::QosPolicy,
    publication::{DataWriter, DataWriterProxyCommand},
    topic::Topic,
    wires::WireFactory,
};

#[derive(Clone)]
pub struct Publisher {
    guid: Guid,
    qos: QosPolicy,
    wire_factory: WireFactory,
    endpoint_lifecycle_sender: Sender<EndpointLifecycleCommand>,
    writer_entity_identifier: Arc<Mutex<EntityIdentifier>>,
}

impl Publisher {
    pub(crate) fn new(
        guid: Guid,
        default_unicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        endpoint_lifecycle_sender: Sender<EndpointLifecycleCommand>,
        writer_entity_identifier: Arc<Mutex<EntityIdentifier>>,
        qos: QosPolicy,
        wire_factory: WireFactory,
        config: Configuration,
    ) -> Self {
        unimplemented!()
    }

    pub fn get_guid_prefix(&self) -> GuidPrefix {
        self.guid.get_guid_prefix()
    }

    pub fn get_entity_id(&self) -> EntityId {
        self.guid.get_entity_id()
    }

    pub async fn create_datawriter<T>(
        &mut self,
        topic: &Topic<T>,
        qos: &QosPolicy,
    ) -> Result<DataWriter<T>, DdsError>
    where
        T: Serialize + Keyed + 'static,
    {
        let writer_key = self.writer_entity_identifier.lock().unwrap().get_new_key();
        let writer_id = if matches!(topic.topic_kind, TopicKind::WithKey) {
            EntityId::writer_with_key(writer_key.0)
        } else {
            EntityId::writer_no_key(writer_key.0)
        };
        let writer_guid = Guid::new(self.guid.get_guid_prefix(), writer_id);

        let datawriter =
            DataWriter::new(writer_guid, *qos, self.endpoint_lifecycle_sender.clone()).await;

        let localhost_wire = self
            .wire_factory
            .build_user_wire(Some("127.0.0.1".parse().unwrap()))
            .unwrap();
        let unicast_wire = self.wire_factory.build_user_wire(None).unwrap();

        datawriter.incomming_task(localhost_wire);
        datawriter.incomming_task(unicast_wire);

        // let old = self
        //     .inner
        //     .lock()
        //     .await
        //     .writers
        //     .insert(writer.get_guid().get_entity_id(), writer.duplicate());

        // assert!(old.is_none());

        Ok(datawriter)
    }
}
