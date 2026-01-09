use bytes::BytesMut;
use chrono::format::Item;

use std::{
    fmt::{Debug, Display},
    hash::Hash,
    mem,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::{Deref, DerefMut},
    slice::Iter,
    sync::Arc,
};

use async_trait::async_trait;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{net::UdpSocket, sync::Mutex};
use tokio_util::codec::{Decoder, Encoder};
use troc_core::{Locator, LocatorList, Message};

use crate::domain::UdpHelper;

use super::{TransmissionKind, Wired};

pub struct Wire(Box<dyn Wired>);

impl Wire {
    pub fn new(wired: Box<dyn Wired>) -> Self {
        Self(wired)
    }
}

impl Deref for Wire {
    type Target = Box<dyn Wired + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Wire {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for Wire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Wire").field(&self.0.locator()).finish()
    }
}

impl Display for Wire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Wire({})", &self.0.locator()))
    }
}

#[derive(Default)]
pub(crate) struct WireList(Vec<Wire>);

impl WireList {
    pub(crate) fn new(wires: Vec<Wire>) -> Self {
        Self(wires)
    }

    pub fn merge(mut self, mut wire_list: WireList) -> Self {
        self.0.append(&mut wire_list.0);
        self
    }

    pub fn extract_locators(&self) -> LocatorList {
        let locators = self.0.iter().map(|w| w.locator()).collect::<Vec<_>>();
        LocatorList::new(locators)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Wire> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Wire> {
        self.0.iter_mut()
    }

    pub fn duplicate(&self) -> Self {
        let wires = self
            .0
            .iter()
            .map(|w| Wire(w.duplicate()))
            .collect::<Vec<Wire>>();
        Self(wires)
    }

    pub fn pop(&mut self) -> Option<Wire> {
        self.0.pop()
    }
}

impl IntoIterator for WireList {
    type Item = Wire;

    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// #[cfg(test)]
// pub(crate) mod tests {
//     use crate::behavior::MockWired;
//     use crate::tests::build_data;

//     use super::{Wire, WireList, Wired};
//     use common::{
//         messages::MessageFactory,
//         types::{
//             ContentNature, ENTITYID_UNKOWN, GuidPrefix, Locator, LocatorKind, SequenceNumber,
//             SerializedData,
//         },
//     };
//     use mockall::predicate;
//     use rstest::{fixture, rstest};

//     #[rstest]
//     #[tokio::test]
//     async fn test_mock(
//         #[from(build_data)]
//         #[with(10)]
//         data: Vec<u8>,
//     ) {
//         let mut factory = MessageFactory::new(GuidPrefix::default());
//         let msg = factory
//             .message()
//             .reader(ENTITYID_UNKOWN)
//             .writer(ENTITYID_UNKOWN)
//             .data(
//                 ContentNature::Data,
//                 SequenceNumber::default(),
//                 None,
//                 Some(SerializedData::new(&data)),
//             )
//             .build();

//         let mut mock = MockWired::new();
//         mock.expect_send()
//             .with(predicate::eq(msg.clone()))
//             .times(1)
//             .returning(|_msg| Ok(()));

//         let mut wire_mock: Box<dyn Wired> = Box::new(mock);
//         let res = wire_mock.send(&msg).await;

//         assert!(res.is_ok())
//     }

//     #[fixture]
//     pub fn build_wire_list_one_wired_mock(
//         #[default(MockWired::new())] wired: MockWired,
//     ) -> WireList {
//         WireList::new(vec![Wire::new(Box::new(wired))])
//     }

//     #[fixture]
//     pub fn build_wire_list_two_wired_mock(
//         #[default(MockWired::new())] wired_0: MockWired,
//         #[default(MockWired::new())] wired_1: MockWired,
//     ) -> WireList {
//         WireList::new(vec![
//             Wire::new(Box::new(wired_0)),
//             Wire::new(Box::new(wired_1)),
//         ])
//     }

//     #[fixture]
//     fn build_unicast_locator() -> Locator {
//         Locator::new(
//             LocatorKind::UdpV4,
//             [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
//             9000,
//         )
//     }
// }
