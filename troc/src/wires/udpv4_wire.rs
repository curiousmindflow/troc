use bytes::BytesMut;
use chrono::format::Item;
use governor::{
    Quota, RateLimiter,
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
};
use itertools::Itertools;
use tracing::{Level, event, instrument};

use std::{
    hash::Hash,
    mem,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroU32,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{net::UdpSocket, sync::Mutex, time::sleep};
use tokio_util::codec::{Decoder, Encoder};
use troc_core::{
    Guid, Locator, LocatorList, Message, SequenceNumber, SubmessageContent,
    submessage_kind::SubmessageKind,
};

use crate::domain::UdpHelper;

use super::{TransmissionDirection, TransmissionKind, WireError, Wired};

pub(crate) struct UdpV4Wire {
    socket: UdpSocket,
    locator: Locator,
    buffer: Vec<u8>,
    _kind: TransmissionKind,
    direction: TransmissionDirection,
    bucket: RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
}

impl UdpV4Wire {
    pub fn new_listener(locator: &Locator, reuse: bool) -> Result<Self, WireError> {
        let sockaddr = UdpHelper::get_socket_addr(locator);
        let socket = UdpV4Wire::common_socket_setup(sockaddr, reuse)?;
        let socket = UdpSocket::from_std(socket.into()).unwrap();
        let wire = Self {
            socket,
            locator: *locator,
            buffer: Vec::with_capacity(64 * 1024),
            _kind: TransmissionKind::ToMany,
            direction: TransmissionDirection::Listener,
            bucket: RateLimiter::direct(Quota::per_second(NonZeroU32::new(1).unwrap())),
        };
        Ok(wire)
    }

    pub fn new_sender(locator: &Locator, reuse: bool) -> Result<Self, WireError> {
        let sockaddr = UdpHelper::get_socket_addr(locator);
        let bind_addr: SocketAddr = format!("{}:{}", sockaddr.ip(), 0).parse().unwrap();

        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        socket.set_reuse_address(reuse).unwrap();
        socket.set_nonblocking(true).unwrap();

        if bind_addr.ip().is_multicast() {
            socket.bind(&SockAddr::from(bind_addr)).unwrap();

            let IpAddr::V4(ipv4_sockaddr_connect) = bind_addr.ip() else {
                panic!()
            };

            socket
                .join_multicast_v4(&ipv4_sockaddr_connect, &Ipv4Addr::new(0, 0, 0, 0))
                .unwrap();
            socket.set_multicast_loop_v4(true).unwrap();
        } else {
            let sockaddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
            let sockaddr = SockAddr::from(sockaddr);
            socket.bind(&sockaddr).unwrap();
        }

        socket.connect(&SockAddr::from(sockaddr)).unwrap();
        let socket = UdpSocket::from_std(socket.into()).unwrap();
        let bucket = RateLimiter::direct(Quota::per_second(
            NonZeroU32::new(80 * 1024 * 1024).unwrap(),
        ));
        let wire = Self {
            socket,
            locator: *locator,
            buffer: Vec::with_capacity(64 * 1024),
            _kind: TransmissionKind::ToMany,
            direction: TransmissionDirection::Sender,
            bucket,
        };
        Ok(wire)
    }

    fn common_socket_setup(sockaddr: SocketAddr, reuse: bool) -> Result<Socket, WireError> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        socket.set_reuse_address(reuse)?;
        socket.set_nonblocking(true)?;
        socket.bind(&SockAddr::from(sockaddr))?;

        if sockaddr.ip().is_multicast() {
            let IpAddr::V4(ipv4_sockaddr_connect) = sockaddr.ip() else {
                panic!()
            };
            socket.join_multicast_v4(&ipv4_sockaddr_connect, &Ipv4Addr::new(0, 0, 0, 0))?;
            socket.set_multicast_loop_v4(true)?;
        }

        Ok(socket)
    }
}

#[async_trait]
impl Wired for UdpV4Wire {
    #[instrument(level = "TRACE", skip_all)]
    async fn recv(&mut self) -> Result<BytesMut, WireError> {
        loop {
            self.socket.readable().await?;

            match self.socket.try_recv_buf(&mut self.buffer) {
                Ok(nb_bytes) => {
                    if nb_bytes == 0 {
                        event!(Level::WARN, "Udp packet is empty");
                        continue;
                    }
                    let msg = BytesMut::from_iter(&self.buffer);
                    self.buffer.clear();
                    break Ok(msg);
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::WouldBlock => continue,
                    _ => {
                        self.buffer.clear();
                        break Err(WireError::ReceptionError(e.to_string()));
                    }
                },
            }
        }
    }

    #[instrument(level = "TRACE", skip_all)]
    async fn send(&mut self, msg: BytesMut) -> Result<(), WireError> {
        loop {
            self.socket.writable().await?;

            // self.bucket
            //     .until_n_ready(NonZeroU32::new(msg.len() as u32).unwrap())
            //     .await
            //     .unwrap();
            // sleep(Duration::from_micros(50)).await;

            match self.socket.try_send(&msg) {
                Ok(_nb_bytes) => {
                    self.buffer.clear();
                    break Ok(());
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    self.buffer.clear();
                    break Err(WireError::SendError(e.to_string()));
                }
            }
        }
    }

    fn transmission_kind(&self) -> TransmissionKind {
        self._kind
    }

    fn transmission_direction(&self) -> TransmissionDirection {
        self.direction
    }

    fn locator(&self) -> Locator {
        self.locator
    }

    fn duplicate(&self) -> Box<dyn Wired> {
        let wire = match self.transmission_direction() {
            TransmissionDirection::Listener => UdpV4Wire::new_listener(&self.locator(), true),
            TransmissionDirection::Sender => UdpV4Wire::new_sender(&self.locator(), true),
        };
        let wire = wire.unwrap();
        Box::new(wire)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use chrono::Utc;
//     use common::{
//         messages::MessageFactory,
//         types::{
//             ContentNature, Count, EntityId, GuidPrefix, Locator, LocatorKind, SequenceNumber,
//             SerializedData, Timestamp,
//         },
//     };
//     use rstest::rstest;
//     use tokio::sync::Notify;

//     use crate::behavior::{Wired, wires::udpv4_wire::UdpV4Wire};

//     #[rstest]
//     #[tokio::test]
//     async fn exchange() {
//         let locator = Locator::new(
//             LocatorKind::UdpV4,
//             [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 224, 0, 0, 0],
//             9000,
//         );
//         let expected_msg = MessageFactory::new(GuidPrefix::default())
//             .message()
//             .reader(EntityId::default())
//             .writer(EntityId::default())
//             .info_timestamp(Some(Timestamp::from_datetime(Utc::now())))
//             .data(
//                 ContentNature::Data,
//                 SequenceNumber::default(),
//                 None,
//                 Some(SerializedData::new(&[0u8; 10])),
//             )
//             .heartbeat(
//                 true,
//                 false,
//                 SequenceNumber(0),
//                 SequenceNumber(1),
//                 Count::new(0),
//                 None,
//             )
//             .build();

//         let mut sender = UdpV4Wire::new_sender(&locator, true).unwrap();
//         let mut listener = UdpV4Wire::new_listener(&locator, true).unwrap();

//         let notifier = Arc::new(Notify::new());
//         let handle = {
//             let notifier = notifier.clone();
//             tokio::spawn(async move {
//                 notifier.notify_one();
//                 listener.recv().await.unwrap()
//             })
//         };

//         notifier.notified().await;
//         sender.send(&expected_msg).await.unwrap();

//         let actual_msg = handle.await.unwrap();

//         assert_eq!(actual_msg, expected_msg);
//     }
// }
