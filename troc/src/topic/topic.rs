use std::marker::PhantomData;

use troc_core::TopicKind;

use crate::infrastructure::QosPolicy;

#[derive(Debug)]
pub struct Topic<T> {
    pub(crate) topic_name: String,
    pub(crate) type_name: String,
    pub(crate) qos: QosPolicy,
    pub(crate) topic_kind: TopicKind,
    _phantom: PhantomData<T>,
}

impl<T> Topic<T> {
    pub(crate) fn new(
        topic_name: impl AsRef<str>,
        type_name: impl AsRef<str>,
        qos: &QosPolicy,
        topic_kind: TopicKind,
    ) -> Self {
        let topic_name = topic_name.as_ref().to_string();
        let type_name = type_name.as_ref().to_string();
        Self {
            topic_name,
            type_name,
            qos: *qos,
            topic_kind,
            _phantom: PhantomData,
        }
    }

    pub fn topic_name(&self) -> &str {
        self.topic_name.as_str()
    }

    pub fn type_name(&self) -> &str {
        self.type_name.as_str()
    }
}
