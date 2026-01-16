use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use troc::cdr::{CdrLe, Infinite};
use troc::{
    DDSType, DataReader, DataWriter, DomainParticipant, DurationKind, HistoryQosPolicy,
    KeyCalculationError, Keyed, Publisher, ReliabilityKind, ReliabilityQosPolicy, Subscriber,
    TopicKind, cdr,
};
use troc::{DomainParticipantBuilder, SerializedData};

pub static O: usize = 1;
pub static K: usize = 1024 * O;
pub static M: usize = 1024 * K;

pub async fn one_writer_one_reader_dds_exchange(
    dds_bundle: &mut SimpleDDSBundle,
    msg: BenchMessage,
) -> BenchMessage {
    dds_bundle.get_writer().write(msg).await.unwrap();
    let msg = dds_bundle.get_reader().read_next_sample().await.unwrap();
    msg.take_data().unwrap()
}

pub async fn one_writer_two_readers_dds_exchange(
    dds_bundle: &mut OneWriterManyReaderDDSBundle,
    msg: BenchMessage,
) {
    dds_bundle.get_writer().write(msg).await.unwrap();
    dds_bundle.get_reader_0().read_next_sample().await.unwrap();
    dds_bundle.get_reader_1().read_next_sample().await.unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, DDSType)]
pub struct BenchMessage {
    payload: Vec<u8>,
}

impl BenchMessage {
    pub fn new(payload_size: usize) -> Self {
        let payload = std::iter::repeat_n(0x8Fu8, payload_size).collect::<Vec<u8>>();
        Self { payload }
    }
}

pub struct SimpleDDSBundle {
    _alpha_p: DomainParticipant,
    _alpha_sub: Subscriber,
    alpha_reader: DataReader<BenchMessage>,
    _beta_p: DomainParticipant,
    _beta_pub: Publisher,
    beta_writer: DataWriter<BenchMessage>,
}

impl SimpleDDSBundle {
    pub async fn new(
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        topic_name_suffix: &str,
        history_qos: HistoryQosPolicy,
    ) -> Self {
        let reliability = if matches!(reliability_level, ReliabilityKind::BestEffort) {
            ReliabilityQosPolicy::BestEffort
        } else {
            ReliabilityQosPolicy::Reliable {
                max_blocking_time: Default::default(),
            }
        };

        let mut alpha_p = DomainParticipantBuilder::new().with_domain(0).build().await;
        let alpha_qos = alpha_p
            .create_qos_builder()
            .reliability(reliability)
            .history(history_qos)
            .build();
        let alpha_topic = alpha_p.create_topic(
            format!("/{topic_name_suffix}_bench"),
            "BenchMessage",
            &alpha_qos,
            topic_kind,
        );
        let mut alpha_sub = alpha_p.create_subscriber(&alpha_qos).await.unwrap();

        let alpha_reader = alpha_sub
            .create_datareader(&alpha_topic, &alpha_qos)
            .await
            .unwrap();
        let mut alpha_reader_listener = alpha_reader.get_listener().await.unwrap();

        let mut beta_p = DomainParticipantBuilder::new().with_domain(0).build().await;
        let beta_qos = beta_p
            .create_qos_builder()
            .reliability(reliability)
            .history(history_qos)
            .build();
        let beta_topic = beta_p.create_topic(
            format!("/{topic_name_suffix}_bench"),
            "BenchMessage",
            &beta_qos,
            topic_kind,
        );
        let mut beta_pub = beta_p.create_publisher(&beta_qos).await.unwrap();

        let beta_writer = beta_pub
            .create_datawriter(&beta_topic, &beta_qos)
            .await
            .unwrap();
        let mut beta_writer_listener = beta_writer.get_listener().await.unwrap();

        let _event = alpha_reader_listener
            .wait_publication_matched(DurationKind::Infinite)
            .await
            .unwrap();

        let _event = beta_writer_listener
            .wait_subscription_matched(DurationKind::Infinite)
            .await
            .unwrap();

        Self {
            _alpha_p: alpha_p,
            _alpha_sub: alpha_sub,
            alpha_reader,
            _beta_p: beta_p,
            _beta_pub: beta_pub,
            beta_writer,
        }
    }

    pub fn get_reader(&mut self) -> &mut DataReader<BenchMessage> {
        &mut self.alpha_reader
    }

    pub fn get_writer(&mut self) -> &mut DataWriter<BenchMessage> {
        &mut self.beta_writer
    }
}

pub struct OneWriterManyReaderDDSBundle {
    _alpha_p: DomainParticipant,
    _alpha_sub: Subscriber,
    alpha_reader: DataReader<BenchMessage>,
    _beta_p: DomainParticipant,
    _beta_pub: Publisher,
    beta_writer: DataWriter<BenchMessage>,
    _gamma_p: DomainParticipant,
    _gamma_sub: Subscriber,
    gamma_reader: DataReader<BenchMessage>,
}

impl OneWriterManyReaderDDSBundle {
    pub async fn new(
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        topic_name_suffix: &str,
        history_qos: HistoryQosPolicy,
    ) -> Self {
        let reliability = if matches!(reliability_level, ReliabilityKind::BestEffort) {
            ReliabilityQosPolicy::BestEffort
        } else {
            ReliabilityQosPolicy::Reliable {
                max_blocking_time: Default::default(),
            }
        };

        let mut alpha_p = DomainParticipantBuilder::new().with_domain(0).build().await;
        let alpha_qos = alpha_p
            .create_qos_builder()
            .reliability(reliability)
            .history(history_qos)
            .build();
        let alpha_topic = alpha_p.create_topic(
            format!("/{topic_name_suffix}_bench"),
            "BenchMessage",
            &alpha_qos,
            topic_kind,
        );
        let mut alpha_sub = alpha_p.create_subscriber(&alpha_qos).await.unwrap();

        let alpha_reader = alpha_sub
            .create_datareader(&alpha_topic, &alpha_qos)
            .await
            .unwrap();
        let mut alpha_reader_listener = alpha_reader.get_listener().await.unwrap();

        let mut beta_p = DomainParticipantBuilder::new().with_domain(0).build().await;
        let beta_qos = beta_p
            .create_qos_builder()
            .reliability(reliability)
            .history(history_qos)
            .build();
        let beta_topic = beta_p.create_topic(
            format!("/{topic_name_suffix}_bench"),
            "BenchMessage",
            &beta_qos,
            topic_kind,
        );
        let mut beta_pub = beta_p.create_publisher(&beta_qos).await.unwrap();

        let beta_writer = beta_pub
            .create_datawriter(&beta_topic, &beta_qos)
            .await
            .unwrap();
        let mut beta_writer_listener = beta_writer.get_listener().await.unwrap();

        let mut gamma_p = DomainParticipantBuilder::new().with_domain(0).build().await;
        let gamma_qos = gamma_p
            .create_qos_builder()
            .reliability(reliability)
            .history(history_qos)
            .build();
        let gamma_topic = gamma_p.create_topic(
            format!("/{topic_name_suffix}_bench"),
            "BenchMessage",
            &gamma_qos,
            topic_kind,
        );
        let mut gamma_sub = gamma_p.create_subscriber(&gamma_qos).await.unwrap();

        let gamma_reader = gamma_sub
            .create_datareader(&gamma_topic, &gamma_qos)
            .await
            .unwrap();
        let mut gamma_reader_listener = gamma_reader.get_listener().await.unwrap();

        let _event = alpha_reader_listener
            .wait_publication_matched(DurationKind::Infinite)
            .await
            .unwrap();

        let _event = gamma_reader_listener
            .wait_publication_matched(DurationKind::Infinite)
            .await
            .unwrap();

        let _event = beta_writer_listener
            .wait_subscription_matched(DurationKind::Infinite)
            .await
            .unwrap();

        let _event = beta_writer_listener
            .wait_subscription_matched(DurationKind::Infinite)
            .await
            .unwrap();

        Self {
            _alpha_p: alpha_p,
            _alpha_sub: alpha_sub,
            alpha_reader,
            _beta_p: beta_p,
            _beta_pub: beta_pub,
            beta_writer,
            _gamma_p: gamma_p,
            _gamma_sub: gamma_sub,
            gamma_reader,
        }
    }

    pub fn get_reader_0(&mut self) -> &mut DataReader<BenchMessage> {
        &mut self.alpha_reader
    }

    pub fn get_reader_1(&mut self) -> &mut DataReader<BenchMessage> {
        &mut self.gamma_reader
    }

    pub fn get_writer(&mut self) -> &mut DataWriter<BenchMessage> {
        &mut self.beta_writer
    }
}

pub async fn create_receiver_udp_socket(addr: &str) -> UdpSocket {
    UdpSocket::bind(addr).await.unwrap()
}

pub async fn create_sender_udp_socket(addr: &str) -> UdpSocket {
    let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    sock.connect(addr).await.unwrap();
    sock
}

pub async fn udp_exchange(
    udp_endpoints: &mut (UdpSocket, UdpSocket),
    msg: BenchMessage,
) -> BenchMessage {
    let payload = cdr::serialize::<_, _, CdrLe>(&msg, Infinite).unwrap();
    let payload_size = payload.len();
    let payload = &payload.chunks(59 * 1024).collect_vec();
    let count = payload.len();

    for packet in payload {
        match udp_endpoints.0.send(packet).await {
            Ok(n) if n != 0 => (),
            _ => panic!(),
        }
    }

    let mut recv_buf = Vec::with_capacity(64 * 1024);
    let mut payload_buf = vec![0u8; payload_size];
    let mut current_pos = 0;

    for _ in 0..count {
        let Ok(n) = udp_endpoints.1.recv_buf(&mut recv_buf).await else {
            panic!()
        };

        let end_pos = current_pos + n;
        payload_buf[current_pos..end_pos].copy_from_slice(&recv_buf[0..n]);
        current_pos += n;
        recv_buf.clear();
    }
    let msg = SerializedData::from_vec(payload_buf);
    cdr::deserialize(msg.get_data()).unwrap()
}
