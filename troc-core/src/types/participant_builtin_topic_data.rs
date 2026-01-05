use binrw::binrw;
use serde::{Deserialize, Serialize};

use super::{builtin_topic_key::BuiltinTopicKey, user_data_qos_policy::UserDataQosPolicy};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ParticipantBuiltinTopicData {
    pub(crate) key: BuiltinTopicKey,
    pub(crate) user_data: UserDataQosPolicy,
}
