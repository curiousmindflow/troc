use std::fmt::{Debug, Display};

use binrw::{BinRead, BinWrite, Endian, binrw};
use serde::{Deserialize, Serialize};

use super::{
    DeadlineQosPolicy, DurabilityQosPolicy, HistoryQosPolicy, InstanceHandle, LifespanQosPolicy,
    LivelinessQosPolicy, ParameterId, ParameterList, ReliabilityQosPolicy, RtpsString,
};

#[derive(
    Default, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, BinRead, BinWrite,
)]
#[bw(map = Self::to_parameter_list)]
#[br(map = |inline_qos| Self::from_parameter_list(inline_qos, Endian::Big))]
pub struct InlineQos {
    pub topic_name: String,
    pub type_name: String,
    pub key_hash: InstanceHandle,
    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub lifespan: LifespanQosPolicy,
    pub history: HistoryQosPolicy,
    pub liveness: LivelinessQosPolicy,
}

impl InlineQos {
    pub fn from_parameter_list(parameter_list: ParameterList, endian: Endian) -> Self {
        let topic_name = parameter_list
            .get_param::<RtpsString>(ParameterId::PID_TOPIC_NAME, endian)
            .unwrap_or_default()
            .into();
        let type_name = parameter_list
            .get_param::<RtpsString>(ParameterId::PID_TYPE_NAME, endian)
            .unwrap_or_default()
            .into();
        let key_hash = parameter_list
            .get_param::<InstanceHandle>(ParameterId::PID_KEY_HASH, endian)
            .unwrap_or_default();
        let durability = parameter_list
            .get_param::<DurabilityQosPolicy>(ParameterId::PID_DURABILITY, endian)
            .unwrap_or_default();
        let deadline = parameter_list
            .get_param::<DeadlineQosPolicy>(ParameterId::PID_DEADLINE, endian)
            .unwrap_or_default();
        let reliability = parameter_list
            .get_param::<ReliabilityQosPolicy>(ParameterId::PID_RELIABILITY, endian)
            .unwrap_or_default();
        let lifespan = parameter_list
            .get_param::<LifespanQosPolicy>(ParameterId::PID_LIFESPAN, endian)
            .unwrap_or_default();
        let history = parameter_list
            .get_param::<HistoryQosPolicy>(ParameterId::PID_HISTORY, endian)
            .unwrap_or_default();
        let liveness = parameter_list
            .get_param::<LivelinessQosPolicy>(ParameterId::PID_LIVELINESS, endian)
            .unwrap_or_default();

        Self {
            topic_name,
            type_name,
            key_hash,
            durability,
            deadline,
            reliability,
            lifespan,
            history,
            liveness,
        }
    }

    pub fn to_parameter_list(&self) -> ParameterList {
        self.to_owned().into()
    }
}

impl From<InlineQos> for ParameterList {
    fn from(value: InlineQos) -> Self {
        let mut param_list = ParameterList::new();
        param_list.set_param(
            ParameterId::PID_TOPIC_NAME,
            RtpsString::new(&value.topic_name),
            Endian::Big,
        );
        param_list.set_param(
            ParameterId::PID_TYPE_NAME,
            RtpsString::new(&value.type_name),
            Endian::Big,
        );
        param_list.set_param(ParameterId::PID_KEY_HASH, value.key_hash, Endian::Big);
        param_list.set_param(ParameterId::PID_DURABILITY, value.durability, Endian::Big);
        param_list.set_param(ParameterId::PID_DEADLINE, value.deadline, Endian::Big);
        param_list.set_param(ParameterId::PID_RELIABILITY, value.reliability, Endian::Big);
        param_list.set_param(ParameterId::PID_LIFESPAN, value.lifespan, Endian::Big);
        param_list.set_param(ParameterId::PID_HISTORY, value.history, Endian::Big);
        param_list.set_param(ParameterId::PID_LIVELINESS, value.liveness, Endian::Big);
        param_list
    }
}

impl From<ParameterList> for InlineQos {
    fn from(value: ParameterList) -> Self {
        InlineQos::from_parameter_list(value, Endian::Big)
    }
}

impl Display for InlineQos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Topic name: {}, Type name: {}, Key hash: {}, Durability: {}, Deadline: {}, Reliability: {}, Lifespan: {}, History: {}, Liveness: {}",
            self.topic_name, self.type_name, self.key_hash, self.durability, self.deadline, self.reliability, self.lifespan, self.history, self.liveness
        ))?;
        Ok(())
    }
}

impl Debug for InlineQos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineQos")
            .field("topic_name", &self.topic_name)
            .field("type_name", &self.type_name)
            .field("key_hash", &self.key_hash)
            .field("durability", &self.durability)
            .field("deadline", &self.deadline)
            .field("reliability", &self.reliability)
            .field("lifespan", &self.lifespan)
            .field("history", &self.history)
            .field("liveness", &self.liveness)
            .finish()
    }
}
