use std::time::Duration;

use rstest::*;
use troc::{DurationKind, K, QosPolicy, ReliabilityQosPolicy, TopicKind};

use crate::fixture::{
    DummyStruct, TwoParticipantsBundle, build_payload, build_qos, setup_log, two_participants,
};

#[rstest]
#[case::rb_wb(ReliabilityQosPolicy::BestEffort, ReliabilityQosPolicy::BestEffort)]
#[case::rb_wr(ReliabilityQosPolicy::BestEffort, ReliabilityQosPolicy::Reliable { max_blocking_time: Default::default() })]
#[case::rr_wr(ReliabilityQosPolicy::Reliable { max_blocking_time: Default::default() }, ReliabilityQosPolicy::Reliable { max_blocking_time: Default::default() })]
#[timeout(Duration::from_secs(10))]
#[tokio::test]
async fn endpoints_have_matched(
    #[from(setup_log)] _setup_log: (),
    #[case] _reader_reliability: ReliabilityQosPolicy,
    #[case] _writer_reliability: ReliabilityQosPolicy,
    #[values(1, 10)] exchange_count: usize,
    #[values(4, 70*K)] _payload_size: u32,
    #[from(build_qos)]
    #[with(_reader_reliability)]
    _reader_qos: QosPolicy,
    #[from(build_qos)]
    #[with(_writer_reliability)]
    _writer_qos: QosPolicy,
    #[with(
        "comm/reliability/exchange",
        TopicKind::NoKey,
        _reader_qos,
        _writer_qos
    )]
    #[future]
    two_participants: TwoParticipantsBundle,
    #[from(build_payload)]
    #[with(_payload_size)]
    payload: Vec<u8>,
) {
    let mut bundle = two_participants.await;

    let expected_msg = DummyStruct::new(0, &payload);

    let mut writer_listener = bundle.alpha_writer.get_listener().await.unwrap();
    writer_listener
        .wait_subscription_matched(DurationKind::Infinite)
        .await
        .unwrap();

    let mut reader_listener = bundle.beta_reader.get_listener().await.unwrap();
    reader_listener
        .wait_publication_matched(DurationKind::Infinite)
        .await
        .unwrap();

    for _ in 0..exchange_count {
        bundle
            .alpha_writer
            .write(expected_msg.clone())
            .await
            .unwrap();
        let sample = bundle.beta_reader.read_next_sample().await.unwrap();
        let actual_msg = sample.take_data().unwrap();
        assert_eq!(actual_msg, expected_msg)
    }
}

// FIXME: this test case should be ok, but now it doesn't pass because Durability QoS is not implemented
// #[rstest]
// #[tokio::test]
// #[serial]
// async fn writer_besteffort_reader_besteffort_late_join(
//     _setup_log: (),
//     #[from(build_qos)]
//     #[with(ReliabilityQosPolicy::BestEffort)]
//     qos: QosPolicy,
//     #[from(build_payload)]
//     #[with(4)]
//     payload: Vec<u8>,
// ) {
//     let mut alpha_domain_participant = DomainParticipant::new_with_name(0, "alpha").await;
//     let mut beta_domain_participant = DomainParticipant::new_with_name(0, "beta").await;

//     let topic_name =
//         build_test_topic("comm/reliability/writer_besteffort_reader_besteffort_late_join");
//     let topic =
//         alpha_domain_participant.create_topic(topic_name, "DummyStruct", &qos, TopicKind::NoKey);

//     let mut beta_subscriber = alpha_domain_participant
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let mut alpha_publisher = beta_domain_participant
//         .create_publisher(&qos)
//         .await
//         .unwrap();

//     let mut alpha_writer = alpha_publisher
//         .create_datawriter_with_name::<DummyStruct>(&topic, &qos, "TestWriter")
//         .await
//         .unwrap();

//     let expected_msg = DummyStruct::new(0, &payload);

//     for _ in 0..10 {
//         alpha_writer.write(expected_msg.clone()).await.unwrap();
//     }

//     sleep(Duration::from_secs(1)).await;

//     let mut beta_reader = beta_subscriber
//         .create_datareader_with_name::<DummyStruct>(&topic, &qos, "TestReader")
//         .await
//         .unwrap();

//     match beta_reader
//         .read_next_sample_timeout(Duration::from_secs(5))
//         .await
//     {
//         Ok(_) => panic!("reader is not expected to read any message"),
//         Err(e) => {
//             assert!(matches!(e, DdsError::Timeout { .. }));
//         }
//     }
// }

// TODO:
// # writer_reliable_reader_besteffort_late_join
// ## initial:
// - writer created
// - no reader
// ## scenario
// - write N samples
// - create Reader
// - read
// ## success condition:
// - reader should read 0 samples

// TODO:
// # writer_reliable_reader_reliable_late_join
// ## initial:
// - writer created
// - no reader
// ## scenario
// - write N samples
// - create Reader
// - read
// ## success condition:
// - reader must read N samples

// TODO:
// # writer_reliable_reader_reliable_late_join_frags
// ## initial:
// - writer created
// - no reader
// ## scenario
// - write N samples
// - create Reader
// - read
// ## success condition:
// - reader must read N samples
