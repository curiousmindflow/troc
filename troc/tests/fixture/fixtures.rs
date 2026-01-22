#![allow(dead_code)]

use std::{str::FromStr, time::Duration};

use crate::fixture::DummyStruct;
use rstest::fixture;
use tracing::{Level, event, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use troc::{
    Configuration, DataReader, DataWriter, DeadlineQosPolicy, DomainParticipant,
    DomainParticipantBuilder, DomainTag, DurabilityQosPolicy, DurationKind, EntityId, Guid,
    GuidPrefix, HistoryQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, Publisher, QosPolicy,
    QosPolicyBuilder, ReliabilityQosPolicy, Subscriber, TopicKind, VendorId,
};

pub struct TwoParticipantsBundle {
    pub alpha_domain_participant: DomainParticipant,
    pub beta_domain_participant: DomainParticipant,
    pub beta_publisher: Publisher,
    pub alpha_subscriber: Subscriber,
    pub beta_writer: DataWriter<DummyStruct>,
    pub alpha_reader: DataReader<DummyStruct>,
}

pub struct ThreeParticipantsBundle {
    pub alpha_domain_participant: DomainParticipant,
    pub alpha_publisher: Publisher,
    pub alpha_subscriber: Subscriber,
    pub alpha_writer: DataWriter<DummyStruct>,
    pub alpha_reader: DataReader<DummyStruct>,
    pub beta_domain_participant: DomainParticipant,
    pub beta_publisher: Publisher,
    pub beta_subscriber: Subscriber,
    pub beta_writer: DataWriter<DummyStruct>,
    pub beta_reader: DataReader<DummyStruct>,
    pub gamma_domain_participant: DomainParticipant,
    pub gamma_publisher: Publisher,
    pub gamma_subscriber: Subscriber,
    pub gamma_writer: DataWriter<DummyStruct>,
    pub gamma_reader: DataReader<DummyStruct>,
}

#[fixture]
#[once]
pub fn setup_log() {
    let filter_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::TRACE.into())
        .from_env_lossy()
        .add_directive("neli=warn".parse().unwrap());

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_target(true);

    let registry = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(filter_layer);

    registry.try_init().unwrap();
}

pub fn build_test_topic(name: &str) -> String {
    format!("/tests/{name}")
}

#[fixture]
pub fn get_unique_id() -> String {
    let process_name = std::env::args().next().unwrap();
    let thread_name = std::thread::current()
        .name()
        .unwrap_or("unknown")
        .to_string();
    format!("{process_name}:::{thread_name}")
}

#[fixture]
pub fn get_guid(#[default(0)] offset: u8) -> Guid {
    Guid::new(
        GuidPrefix::from(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, offset],
            VendorId::default(),
        ),
        EntityId::default(),
    )
}

#[fixture]
pub async fn two_participants(
    #[default("")] topic_name: impl AsRef<str>,
    #[default(TopicKind::NoKey)] topic_kind: TopicKind,
    #[default(QosPolicy::default())] reader_qos: QosPolicy,
    #[default(QosPolicy::default())] writer_qos: QosPolicy,
    #[default(0)] alpha_domain_id: u32,
    #[default(0)] beta_domain_id: u32,
    #[from(get_guid)] alpha_guid: Guid,
    #[from(get_guid)]
    #[with(1)]
    beta_guid: Guid,
    #[from(get_unique_id)] unique_id: String,
) -> TwoParticipantsBundle {
    let mut configuration = Configuration::default();
    configuration.global.domain_tag = DomainTag::from_str(&unique_id).unwrap();
    configuration.discovery.announcement_period = Duration::from_secs(1);
    configuration.discovery.lease_duration = Duration::from_secs(3);

    let mut alpha_domain_participant = DomainParticipantBuilder::new()
        .with_guid(alpha_guid)
        .with_domain(alpha_domain_id)
        .with_config(configuration.clone())
        .build()
        .await;

    let mut beta_domain_participant = DomainParticipantBuilder::new()
        .with_guid(beta_guid)
        .with_domain(beta_domain_id)
        .with_config(configuration.clone())
        .build()
        .await;

    let topic_name = build_test_topic(topic_name.as_ref());
    let topic =
        alpha_domain_participant.create_topic(topic_name, "DummyStruct", &reader_qos, topic_kind);

    let mut alpha_subscriber = alpha_domain_participant
        .create_subscriber(&reader_qos)
        .await
        .unwrap();
    let mut beta_publisher = beta_domain_participant
        .create_publisher(&writer_qos)
        .await
        .unwrap();

    let alpha_reader = alpha_subscriber
        .create_datareader::<DummyStruct>(&topic, &reader_qos)
        .await
        .unwrap();

    let beta_writer = beta_publisher
        .create_datawriter::<DummyStruct>(&topic, &writer_qos)
        .await
        .unwrap();

    event!(Level::INFO, "### two_participants fixture done");

    TwoParticipantsBundle {
        alpha_domain_participant,
        beta_domain_participant,
        beta_publisher,
        alpha_subscriber,
        beta_writer,
        alpha_reader,
    }
}

#[fixture]
pub async fn three_participants(
    #[default("")] topic_name: &str,
    #[default(0)] domain_id: u32,
    #[from(get_unique_id)] unique_id: String,
) -> ThreeParticipantsBundle {
    let mut configuration = Configuration::default();
    configuration.global.domain_tag = DomainTag::from_str(&unique_id).unwrap();

    let mut alpha_domain_participant = DomainParticipantBuilder::new()
        .with_domain(domain_id)
        .with_config(configuration.clone())
        .build()
        .await;

    let qos = alpha_domain_participant
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .history(HistoryQosPolicy::KeepLast { depth: 1 })
        .build();

    let topic_name = build_test_topic(topic_name);
    let topic =
        alpha_domain_participant.create_topic(topic_name, "DummyStruct", &qos, TopicKind::NoKey);

    let mut alpha_subscriber = alpha_domain_participant
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut alpha_publisher = alpha_domain_participant
        .create_publisher(&qos)
        .await
        .unwrap();

    let alpha_reader = alpha_subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();
    let mut alpha_reader_listener = alpha_reader.get_listener().await.unwrap();

    let alpha_writer = alpha_publisher
        .create_datawriter::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();
    let mut alpha_writer_listener = alpha_writer.get_listener().await.unwrap();

    let mut beta_domain_participant = DomainParticipantBuilder::new()
        .with_domain(domain_id)
        .with_config(configuration.clone())
        .build()
        .await;

    let mut beta_subscriber = beta_domain_participant
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut beta_publisher = beta_domain_participant
        .create_publisher(&qos)
        .await
        .unwrap();

    let beta_reader = beta_subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();
    let mut beta_reader_listener = beta_reader.get_listener().await.unwrap();

    let beta_writer = beta_publisher
        .create_datawriter::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();
    let mut beta_writer_listener = beta_writer.get_listener().await.unwrap();

    let mut gamma_domain_participant = DomainParticipantBuilder::new()
        .with_domain(domain_id)
        .with_config(configuration)
        .build()
        .await;

    let mut gamma_subscriber = gamma_domain_participant
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut gamma_publisher = gamma_domain_participant
        .create_publisher(&qos)
        .await
        .unwrap();

    let gamma_reader = gamma_subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();
    let mut gamma_reader_listener = gamma_reader.get_listener().await.unwrap();

    let gamma_writer = gamma_publisher
        .create_datawriter::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();
    let mut gamma_writer_listener = gamma_writer.get_listener().await.unwrap();

    let _event = alpha_reader_listener
        .wait_publication_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let _event = alpha_reader_listener
        .wait_publication_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let _event = alpha_writer_listener
        .wait_subscription_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let _event = alpha_writer_listener
        .wait_subscription_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let _event = beta_reader_listener
        .wait_publication_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let _event = beta_reader_listener
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

    let _event = gamma_reader_listener
        .wait_publication_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let _event = gamma_reader_listener
        .wait_publication_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let _event = gamma_writer_listener
        .wait_subscription_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let _event = gamma_writer_listener
        .wait_subscription_matched(DurationKind::Infinite)
        .await
        .unwrap();

    event!(Level::INFO, "### three_participants fixture done");

    ThreeParticipantsBundle {
        alpha_domain_participant,
        alpha_publisher,
        alpha_subscriber,
        alpha_writer,
        alpha_reader,
        beta_domain_participant,
        beta_publisher,
        beta_subscriber,
        beta_writer,
        beta_reader,
        gamma_domain_participant,
        gamma_publisher,
        gamma_subscriber,
        gamma_writer,
        gamma_reader,
    }
}

#[fixture]
pub fn build_qos(
    #[default(ReliabilityQosPolicy::Reliable { max_blocking_time: Default::default() })]
    reliability: ReliabilityQosPolicy,
    #[default(HistoryQosPolicy::KeepAll)] history: HistoryQosPolicy,
    #[default(DurabilityQosPolicy::default())] durability: DurabilityQosPolicy,
    #[default(LifespanQosPolicy::default())] lifespan: LifespanQosPolicy,
    #[default(LivelinessQosPolicy::default())] liveness: LivelinessQosPolicy,
    #[default(DeadlineQosPolicy::default())] deadline: DeadlineQosPolicy,
) -> QosPolicy {
    QosPolicyBuilder::new()
        .reliability(reliability)
        .history(history)
        .durability(durability)
        .lifespan(lifespan)
        .liveness(liveness)
        .deadline(deadline)
        .build()
}

#[fixture]
pub fn build_payload(#[default(4)] size: u32) -> Vec<u8> {
    std::iter::repeat_n(7u8, size as usize).collect::<Vec<u8>>()
}

// /// Test with multiple writers and a single reader
// #[fixture]
// pub async fn multiple_writers_single_reader(
//     #[default("/test")] topic_name: &'static str,
//     #[default(3)] number_w: u32,
//     #[default(ReliabilityQosPolicy::default())] reliability: ReliabilityQosPolicy,
//     #[default(HistoryQosPolicy::default())] historycache: HistoryQosPolicy,
//     #[default(DurabilityQosPolicy::default())] durability: DurabilityQosPolicy,
//     #[default(LifespanQosPolicy::default())] _lifespan: LifespanQosPolicy,
//     #[default(LivelinessQosPolicy::default())] _liveliness: LivelinessQosPolicy,
//     #[default(DeadlineQosPolicy::default())] _deadline: DeadlineQosPolicy,
// ) -> (
//     DataReader<DummyStruct>,
//     Vec<(DomainParticipant, Publisher, DataWriter<DummyStruct>)>,
// ) {
//     let mut domain_participant_read = DomainParticipant::new_with_name(0, "alpha").await;
//     let qos = domain_participant_read
//         .create_qos_builder()
//         .reliability(reliability)
//         .history(historycache)
//         .durability(durability)
//         .build();

//     let topic = domain_participant_read.create_topic(topic_name, "String", &qos, TopicKind::NoKey);
//     let mut subscriber = domain_participant_read
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let data_reader = subscriber
//         .create_datareader::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     let mut writers = Vec::new();
//     for i in 0..number_w {
//         println!("Creating writer participant {}", i);
//         let mut domain_participant_write =
//             DomainParticipant::new_with_name(0, &format!("writer_{}", i)).await;
//         let mut publisher = domain_participant_write
//             .create_publisher(&qos)
//             .await
//             .unwrap();
//         let data_writer = publisher
//             .create_datawriter::<DummyStruct>(&topic, &qos)
//             .await
//             .unwrap();
//         writers.push((domain_participant_write, publisher, data_writer));
//     }
//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;

//     (data_reader, writers)
// }

// /// Test with multiple writers and a single reader
// #[fixture]
// pub async fn multiple_readers_single_writer(
//     #[default("/test")] topic_name: &'static str,
//     #[default(3)] number_w: u32,
//     #[default(ReliabilityQosPolicy::default())] reliability: ReliabilityQosPolicy,
//     #[default(HistoryQosPolicy::default())] historycache: HistoryQosPolicy,
//     #[default(DurabilityQosPolicy::default())] durability: DurabilityQosPolicy,
//     #[default(LifespanQosPolicy::default())] _lifespan: LifespanQosPolicy,
//     #[default(LivelinessQosPolicy::default())] _liveliness: LivelinessQosPolicy,
//     #[default(DeadlineQosPolicy::default())] _deadline: DeadlineQosPolicy,
// ) -> (
//     DataWriter<DummyStruct>,
//     Vec<(DomainParticipant, Subscriber, DataReader<DummyStruct>)>,
// ) {
//     // Create writer participant
//     let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

//     let qos = domain_participant_write
//         .create_qos_builder()
//         .reliability(reliability)
//         .history(historycache)
//         .durability(durability)
//         .build();

//     let topic = domain_participant_write.create_topic(topic_name, "String", &qos, TopicKind::NoKey);
//     let mut publisher = domain_participant_write
//         .create_publisher(&qos)
//         .await
//         .unwrap();
//     let data_writer = publisher
//         .create_datawriter::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Create multiple reader participants (3 readers)
//     let mut readers = Vec::new();
//     for i in 0..number_w {
//         println!("Creating reader participant {}", i);
//         let mut domain_participant_read =
//             DomainParticipant::new_with_name(0, &format!("reader_{}", i)).await;
//         let mut subscriber = domain_participant_read
//             .create_subscriber(&qos)
//             .await
//             .unwrap();
//         let data_reader = subscriber
//             .create_datareader::<DummyStruct>(&topic, &qos)
//             .await
//             .unwrap();
//         readers.push((domain_participant_read, subscriber, data_reader));
//     }
//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;

//     (data_writer, readers)
// }

// /// Test with multiple writers and multiple reader
// #[fixture]
// pub async fn mesh(
//     #[default("/test")] topic_name: &'static str,
//     #[default(ReliabilityQosPolicy::default())] reliability: ReliabilityQosPolicy,
//     #[default(HistoryQosPolicy::default())] historycache: HistoryQosPolicy,
//     #[default(DurabilityQosPolicy::default())] durability: DurabilityQosPolicy,
//     #[default(LifespanQosPolicy::default())] _lifespan: LifespanQosPolicy,
//     #[default(LivelinessQosPolicy::default())] _liveliness: LivelinessQosPolicy,
//     #[default(DeadlineQosPolicy::default())] _deadline: DeadlineQosPolicy,
// ) -> (
//     Vec<(i32, i32, DataWriter<DummyStruct>)>,
//     Vec<(i32, i32, DataReader<DummyStruct>)>,
// ) {
//     println!("Starting many participants discovery test");

//     // Create three participants
//     let mut participants = vec![
//         DomainParticipant::new_with_name(0, "domain_participant_1").await,
//         DomainParticipant::new_with_name(0, "domain_participant_2").await,
//         DomainParticipant::new_with_name(0, "domain_participant_3").await,
//     ];

//     let qos = participants[0]
//         .create_qos_builder()
//         .reliability(reliability)
//         .history(historycache)
//         .durability(durability)
//         .build();

//     // Create topics for each communication path
//     let topic_names = vec![
//         format!("/test{}/mesh_1_2", topic_name),
//         format!("/test{}/mesh_1_3", topic_name),
//         format!("/test{}/mesh_2_3", topic_name),
//         format!("/test{}/mesh_2_1", topic_name),
//         format!("/test{}/mesh_3_1", topic_name),
//         format!("/test{}/mesh_3_2", topic_name),
//     ];

//     // Create endpoints for each participant
//     let mut readers = vec![];
//     let mut writers = vec![];

//     // Participant 1 endpoints
//     let mut subscriber_1 = participants[0].create_subscriber(&qos).await.unwrap();
//     readers.push((
//         1,
//         2,
//         subscriber_1
//             .create_datareader::<DummyStruct>(
//                 &participants[0].create_topic(
//                     topic_names[0].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));
//     readers.push((
//         1,
//         3,
//         subscriber_1
//             .create_datareader::<DummyStruct>(
//                 &participants[0].create_topic(
//                     topic_names[1].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));

//     let mut publisher_1 = participants[0].create_publisher(&qos).await.unwrap();
//     writers.push((
//         1,
//         2,
//         publisher_1
//             .create_datawriter::<DummyStruct>(
//                 &participants[0].create_topic(
//                     topic_names[3].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));
//     writers.push((
//         1,
//         3,
//         publisher_1
//             .create_datawriter::<DummyStruct>(
//                 &participants[0].create_topic(
//                     topic_names[4].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));

//     // Participant 2 endpoints
//     let mut subscriber_2 = participants[1].create_subscriber(&qos).await.unwrap();
//     readers.push((
//         2,
//         1,
//         subscriber_2
//             .create_datareader::<DummyStruct>(
//                 &participants[1].create_topic(
//                     topic_names[3].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));
//     readers.push((
//         2,
//         3,
//         subscriber_2
//             .create_datareader::<DummyStruct>(
//                 &participants[1].create_topic(
//                     topic_names[2].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));

//     let mut publisher_2 = participants[1].create_publisher(&qos).await.unwrap();
//     writers.push((
//         2,
//         1,
//         publisher_2
//             .create_datawriter::<DummyStruct>(
//                 &participants[1].create_topic(
//                     topic_names[0].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));
//     writers.push((
//         2,
//         3,
//         publisher_2
//             .create_datawriter::<DummyStruct>(
//                 &participants[1].create_topic(
//                     topic_names[5].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));

//     // Participant 3 endpoints
//     let mut subscriber_3 = participants[2].create_subscriber(&qos).await.unwrap();
//     readers.push((
//         3,
//         1,
//         subscriber_3
//             .create_datareader::<DummyStruct>(
//                 &participants[2].create_topic(
//                     topic_names[4].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));
//     readers.push((
//         3,
//         2,
//         subscriber_3
//             .create_datareader::<DummyStruct>(
//                 &participants[2].create_topic(
//                     topic_names[5].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));

//     let mut publisher_3 = participants[2].create_publisher(&qos).await.unwrap();
//     writers.push((
//         3,
//         1,
//         publisher_3
//             .create_datawriter::<DummyStruct>(
//                 &participants[2].create_topic(
//                     topic_names[1].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));
//     writers.push((
//         3,
//         2,
//         publisher_3
//             .create_datawriter::<DummyStruct>(
//                 &participants[2].create_topic(
//                     topic_names[2].clone(),
//                     "String",
//                     &qos,
//                     TopicKind::NoKey,
//                 ),
//                 &qos,
//             )
//             .await
//             .unwrap(),
//     ));

//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;

//     (writers, readers)
// }

// #[fixture]
// #[once]
// pub fn setup_log_tokio_console() {
//     console_subscriber::init();
// }

// use opentelemetry::global;
// use tracing::{event, level_filters::LevelFilter, Level};
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// #[fixture]
// // #[once]
// pub fn setup_log_open_telemetry(#[default("test-app")] service_name: &str) {
//     global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

//     let filter_layer = EnvFilter::builder()
//         .with_default_directive(LevelFilter::TRACE.into())
//         .from_env_lossy();

//     let filter_layer = filter_layer.add_directive("neli=warn".parse().unwrap());

//     let stdout_layer = tracing_subscriber::fmt::layer()
//         .with_line_number(true)
//         .with_target(true);

//     let tracer = opentelemetry_jaeger::new_pipeline()
//         .with_service_name(service_name)
//         .install_simple()
//         .unwrap();

//     let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

//     tracing_subscriber::registry()
//         .with(filter_layer)
//         .with(stdout_layer)
//         .with(opentelemetry)
//         .try_init()
//         .unwrap();
// }
