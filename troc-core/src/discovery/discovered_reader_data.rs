use std::fmt::Display;

use crate::{
    common::Error,
    types::{
        EntityId, Guid, InlineQos, Locator, ParameterId, ParameterList, RtpsString, SerializedData,
    },
};
use binrw::Endian;
use serde::{Deserialize, Serialize};

use crate::ReaderProxy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredReaderData {
    pub proxy: ReaderProxy,
    pub params: InlineQos,
}

impl DiscoveredReaderData {
    pub fn from_serialized_data(data: SerializedData) -> Result<Self, Error> {
        let (param_list, endian) = ParameterList::from_serialized_data(data).unwrap();
        let data = Self::from_parameter_list(param_list, endian);
        Ok(data)
    }

    pub fn into_serialized_data(self, endian: Endian) -> Result<SerializedData, Error> {
        let parameter_list = Self::into_parameter_list(self, endian);
        let data = parameter_list.into_serialized_data(endian).unwrap();
        Ok(data)
    }

    fn from_parameter_list(parameter_list: ParameterList, endian: Endian) -> Self {
        let remote_reader_guid = parameter_list
            .get_param::<Guid>(ParameterId::PID_ENDPOINT_GUID, endian)
            .unwrap_or_default();
        let remote_group_entity_id = parameter_list
            .get_param::<EntityId>(ParameterId::PID_GROUP_ENTITY_ID, endian)
            .unwrap_or_default();
        let expects_inline_qos = parameter_list
            .get_param_raw(ParameterId::PID_EXPECTS_INLINE_QOS)
            .map(|v| v[0] != 0)
            .unwrap_or_default();
        let unicast_locator_list = (&*parameter_list
            .get_params::<Locator>(ParameterId::PID_UNICAST_LOCATOR, endian))
            .into();
        let multicast_locator_list = (&*parameter_list
            .get_params::<Locator>(ParameterId::PID_MULTICAST_LOCATOR, endian))
            .into();

        let params = InlineQos::from_parameter_list(parameter_list, endian);

        Self {
            proxy: ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                expects_inline_qos,
                unicast_locator_list,
                multicast_locator_list,
                is_active: true,
                ..Default::default()
            },
            params,
        }
    }

    fn into_parameter_list(self, endian: Endian) -> ParameterList {
        let mut parameter_list = ParameterList::new();
        let inline_param_list = self.params.to_parameter_list();
        parameter_list.merge(&inline_param_list);

        parameter_list.set_param::<RtpsString>(
            ParameterId::PID_TOPIC_NAME,
            self.params.topic_name.into(),
            endian,
        );

        parameter_list.set_param::<RtpsString>(
            ParameterId::PID_TYPE_NAME,
            self.params.type_name.into(),
            endian,
        );

        parameter_list.set_param(
            ParameterId::PID_ENDPOINT_GUID,
            self.proxy.remote_reader_guid,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_GROUP_ENTITY_ID,
            self.proxy.remote_group_entity_id,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_EXPECTS_INLINE_QOS,
            self.proxy.expects_inline_qos as u8,
            endian,
        );

        for locator in self.proxy.unicast_locator_list.iter() {
            parameter_list.set_param(ParameterId::PID_UNICAST_LOCATOR, locator, endian);
        }

        for locator in self.proxy.multicast_locator_list.iter() {
            parameter_list.set_param(ParameterId::PID_MULTICAST_LOCATOR, locator, endian);
        }

        parameter_list
    }
}

impl Display for DiscoveredReaderData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Proxy: {}, Params: {}", self.proxy, self.params))?;
        Ok(())
    }
}
