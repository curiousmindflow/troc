mod cache_change;
mod counter;
mod effect;
mod error;
mod qos_matcher;
mod reader_proxy;
mod writer_proxy;

pub use cache_change::{
    CacheChange, CacheChangeContainer, CacheChangeInfos, FragmentedCacheChange,
};
pub use counter::Counter;
pub use effect::{Effect, EffectConsumption, Effects, TickId};
pub use error::Error;
pub use qos_matcher::QosPolicyConsistencyChecker;
pub use reader_proxy::ReaderProxy;
pub use writer_proxy::WriterProxy;

use crate::{messages::Message, types::LocatorList};
use chrono::Local;

#[derive(Debug)]
pub struct IncommingMessage {
    pub timestamp_millis: i64,
    pub message: Message,
}

impl IncommingMessage {
    pub fn new(message: Message) -> Self {
        let timestamp_millis = Local::now().timestamp_millis();
        Self {
            timestamp_millis,
            message,
        }
    }
}

#[derive(Debug)]
pub struct OutcommingMessage {
    pub timestamp_millis: i64,
    pub message: Message,
    pub locators: LocatorList,
}

impl OutcommingMessage {
    pub fn new(message: Message, locators: LocatorList) -> Self {
        let timestamp_millis = Local::now().timestamp_millis();
        Self {
            timestamp_millis,
            message,
            locators,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use crate::types::{EntityId, Guid, GuidPrefix, Locator, LocatorList, VendorId};
    use rstest::fixture;

    #[fixture]
    pub fn setup_uni_locatorlist() -> LocatorList {
        let locator = Locator::from_str("127.0.0.1:9000:UDPV4").unwrap();
        LocatorList::new(vec![locator])
    }

    #[fixture]
    pub fn setup_reader_0_guid(#[from(setup_guid_prefix)] prefix: GuidPrefix) -> Guid {
        Guid::new(prefix, EntityId::default())
    }

    #[fixture]
    pub fn setup_writer_0_guid(
        #[from(setup_guid_prefix)]
        #[with(1)]
        prefix: GuidPrefix,
    ) -> Guid {
        Guid::new(prefix, EntityId::default())
    }

    #[fixture]
    pub fn setup_guid_prefix(#[default(0)] offset: u8) -> GuidPrefix {
        GuidPrefix::from(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, offset],
            VendorId::default(),
        )
    }
}
