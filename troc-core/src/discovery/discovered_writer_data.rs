use std::fmt::Display;

use crate::{
    common::Error,
    types::{EntityId, Guid, InlineQos, Locator, ParameterId, ParameterList, SerializedData},
};
use binrw::Endian;
use serde::{Deserialize, Serialize};

use crate::WriterProxy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredWriterData {
    pub proxy: WriterProxy,
    pub params: InlineQos,
}

impl DiscoveredWriterData {
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
        let remote_writer_guid = parameter_list
            .get_param::<Guid>(ParameterId::PID_ENDPOINT_GUID, endian)
            .unwrap_or_default();
        let remote_group_entity_id = parameter_list
            .get_param::<EntityId>(ParameterId::PID_GROUP_ENTITY_ID, endian)
            .unwrap_or_default();
        let data_max_size_serialized = parameter_list
            .get_param_raw(ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED)
            .map(|p| {
                let p_to_u32: [u8; 4] = (&p[0..4]).try_into().unwrap();
                match endian {
                    Endian::Little => u32::from_le_bytes(p_to_u32),
                    Endian::Big => u32::from_be_bytes(p_to_u32),
                }
            })
            .unwrap_or_default();
        let unicast_locator_list = (&*parameter_list
            .get_params::<Locator>(ParameterId::PID_UNICAST_LOCATOR, endian))
            .into();
        let multicast_locator_list = (&*parameter_list
            .get_params::<Locator>(ParameterId::PID_MULTICAST_LOCATOR, endian))
            .into();

        let params = InlineQos::from_parameter_list(parameter_list, endian);

        Self {
            proxy: WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list,
                multicast_locator_list,
                data_max_size_serialized,
                ..Default::default()
            },
            params,
        }
    }

    fn into_parameter_list(self, endian: Endian) -> ParameterList {
        let mut parameter_list = ParameterList::new();
        let inline_param_list = self.params.to_parameter_list();
        parameter_list.merge(&inline_param_list);

        parameter_list.set_param(
            ParameterId::PID_ENDPOINT_GUID,
            self.proxy.remote_writer_guid,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_GROUP_ENTITY_ID,
            self.proxy.remote_group_entity_id,
            endian,
        );
        parameter_list.set_param(
            ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED,
            self.proxy.data_max_size_serialized,
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

impl Display for DiscoveredWriterData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Proxy: {}, Params: {}", self.proxy, self.params))?;
        Ok(())
    }
}
