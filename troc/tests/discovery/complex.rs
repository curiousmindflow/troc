//! This module regroup tests that assert the correct behavior of advanced discovery features
//!
//! Here we don't want to test:
//! - data exchange, we just want to assert a communication happened
//! - QoS combination, we just want the default QoS

use std::{str::FromStr, sync::Arc, time::Duration};

use crate::fixture::{
    DummyStruct, ThreeParticipantsBundle, get_unique_id, setup_log, three_participants,
};
use rstest::*;
use tokio::sync::Notify;
use troc::{
    Configuration, DataWriter, DomainParticipantBuilder, DomainTag, QosPolicy, QosPolicyBuilder,
    ReliabilityQosPolicy, TopicKind,
};

#[rstest]
// #[timeout(Duration::from_secs(10))]
#[tokio::test]
async fn discovery_by_exchange_single_thread(
    #[from(setup_log)] _setup_log: (),
    #[from(three_participants)]
    #[with("/discovery/complex/discovery_by_exchange/single_thread")]
    #[future]
    three_participants: ThreeParticipantsBundle,
) {
    let bundle = three_participants.await;
    exchange(bundle).await;
}

#[rstest]
#[timeout(Duration::from_secs(10))]
#[tokio::test(flavor = "multi_thread")]
async fn discovery_by_exchange_multi_thread(
    #[from(setup_log)] _setup_log: (),
    #[from(three_participants)]
    #[with("/discovery/complex/discovery_by_exchange/multi_thread")]
    #[future]
    three_participants: ThreeParticipantsBundle,
) {
    let bundle = three_participants.await;
    exchange(bundle).await;
}

#[rstest]
#[timeout(std::time::Duration::from_secs(15))]
#[tokio::test]
async fn static_discovery_by_exchange_different_topics(
    #[from(get_unique_id)] unique_id: String,
    _setup_log: (),
) {
    let mut configuration = Configuration::default();
    configuration.global.domain_tag = DomainTag::from_str(&unique_id).unwrap();

    let mut alpha_participant = DomainParticipantBuilder::new()
        .with_config(configuration.clone())
        .build()
        .await;

    let mut alpha_subscriber = alpha_participant
        .create_subscriber(&QosPolicy::default())
        .await
        .unwrap();
    let mut alpha_publisher = alpha_participant
        .create_publisher(&QosPolicy::default())
        .await
        .unwrap();

    let mut beta_participant = DomainParticipantBuilder::new()
        .with_config(configuration.clone())
        .build()
        .await;

    let mut beta_subscriber = beta_participant
        .create_subscriber(&QosPolicy::default())
        .await
        .unwrap();
    let mut beta_publisher = beta_participant
        .create_publisher(&QosPolicy::default())
        .await
        .unwrap();

    let qos = QosPolicyBuilder::new()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .build();

    let topic_0 = alpha_participant.create_topic(
        "/test_0",
        "DummyStruct",
        &QosPolicy::default(),
        TopicKind::NoKey,
    );

    let mut alpha_reader_0 = alpha_subscriber
        .create_datareader::<DummyStruct>(&topic_0, &qos)
        .await
        .unwrap();

    let mut beta_writer_0: DataWriter<DummyStruct> = beta_publisher
        .create_datawriter::<DummyStruct>(&topic_0, &qos)
        .await
        .unwrap();

    let topic_1 = alpha_participant.create_topic(
        "/test_1",
        "DummyStruct",
        &QosPolicy::default(),
        TopicKind::NoKey,
    );

    let mut alpha_writer_1 = alpha_publisher
        .create_datawriter::<DummyStruct>(&topic_1, &qos)
        .await
        .unwrap();

    let mut beta_reader_1 = beta_subscriber
        .create_datareader::<DummyStruct>(&topic_1, &qos)
        .await
        .unwrap();

    let topic_2 = alpha_participant.create_topic(
        "/test_2",
        "DummyStruct",
        &QosPolicy::default(),
        TopicKind::NoKey,
    );

    let mut beta_writer_2 = beta_publisher
        .create_datawriter::<DummyStruct>(&topic_2, &qos)
        .await
        .unwrap();

    let mut alpha_reader_2 = alpha_subscriber
        .create_datareader::<DummyStruct>(&topic_2, &qos)
        .await
        .unwrap();

    let topic_3 = alpha_participant.create_topic(
        "/test_3",
        "DummyStruct",
        &QosPolicy::default(),
        TopicKind::NoKey,
    );

    let mut beta_reader_3 = beta_subscriber
        .create_datareader::<DummyStruct>(&topic_3, &qos)
        .await
        .unwrap();

    let mut alpha_writer_3 = alpha_publisher
        .create_datawriter::<DummyStruct>(&topic_3, &qos)
        .await
        .unwrap();

    beta_writer_0.write(DummyStruct::default()).await.unwrap();
    alpha_reader_0.read_next_sample().await.unwrap();

    alpha_writer_1.write(DummyStruct::default()).await.unwrap();
    beta_reader_1.read_next_sample().await.unwrap();

    beta_writer_2.write(DummyStruct::default()).await.unwrap();
    alpha_reader_2.read_next_sample().await.unwrap();

    alpha_writer_3.write(DummyStruct::default()).await.unwrap();
    beta_reader_3.read_next_sample().await.unwrap();
}

#[rstest]
#[timeout(std::time::Duration::from_secs(15))]
#[tokio::test]
async fn dynamic_discovery_by_exchange_different_topics(
    #[from(get_unique_id)] unique_id: String,
    _setup_log: (),
) {
    let mut configuration = Configuration::default();
    configuration.global.domain_tag = DomainTag::from_str(&unique_id).unwrap();

    let mut alpha_participant = DomainParticipantBuilder::new()
        .with_config(configuration.clone())
        .build()
        .await;

    let mut beta_participant = DomainParticipantBuilder::new()
        .with_config(configuration.clone())
        .build()
        .await;

    // SERVER SETUP
    let mut alpha_subscriber = alpha_participant
        .create_subscriber(&QosPolicy::default())
        .await
        .unwrap();
    let mut alpha_publisher = alpha_participant
        .create_publisher(&QosPolicy::default())
        .await
        .unwrap();

    // CLIENT SETUP
    let mut beta_subscriber = beta_participant
        .create_subscriber(&QosPolicy::default())
        .await
        .unwrap();
    let mut beta_publisher = beta_participant
        .create_publisher(&QosPolicy::default())
        .await
        .unwrap();

    let topic_call = alpha_participant.create_topic(
        "/dynamic_scenario_call_test",
        "DummyStruct",
        &QosPolicy::default(),
        TopicKind::NoKey,
    );

    let qos_call = QosPolicyBuilder::new()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .build();

    // CALLING PHASE: CLIENT -> SERVER
    let mut alpha_reader = alpha_subscriber
        .create_datareader::<DummyStruct>(&topic_call, &qos_call)
        .await
        .unwrap();

    let mut beta_writer = beta_publisher
        .create_datawriter::<DummyStruct>(&topic_call, &qos_call)
        .await
        .unwrap();

    let expected_msg = DummyStruct::new(0, &[]);

    let notifier = Arc::new(Notify::new());
    {
        let notifier = notifier.clone();
        let expected_msg = expected_msg.clone();

        tokio::spawn(async move {
            if let Ok(sample) = alpha_reader.read_next_sample().await {
                let actual_msg = sample.take_data().unwrap();
                assert_eq!(actual_msg, expected_msg);

                notifier.notify_one();
            } else {
                panic!()
            }
        });
    }

    beta_writer.write(expected_msg).await.unwrap();

    notifier.notified().await;

    // RESPONSE PHASE: SERVER -> CLIENT
    let topic_answer = alpha_participant.create_topic(
        "/dynamic_scenario_response_test",
        "DummyStruct",
        &QosPolicy::default(),
        TopicKind::NoKey,
    );

    let qos_answer = QosPolicyBuilder::new()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .build();

    let mut alpha_writer = alpha_publisher
        .create_datawriter::<DummyStruct>(&topic_answer, &qos_answer)
        .await
        .unwrap();

    let mut beta_reader = beta_subscriber
        .create_datareader::<DummyStruct>(&topic_answer, &qos_answer)
        .await
        .unwrap();

    let expected_answer_msg = DummyStruct::new(0, &[]);

    let notifier = Arc::new(Notify::new());
    {
        let notifier = notifier.clone();
        let expected_answer_msg = expected_answer_msg.clone();

        tokio::spawn(async move {
            if let Ok(sample) = beta_reader.read_next_sample().await {
                let actual_msg = sample.take_data().unwrap();
                assert_eq!(actual_msg, expected_answer_msg);

                notifier.notify_one();
            } else {
                panic!()
            }
        });
    }

    alpha_writer.write(expected_answer_msg).await.unwrap();
}

async fn exchange(mut bundle: ThreeParticipantsBundle) {
    let expected_msg = DummyStruct::new(0, &[]);

    bundle
        .alpha_writer
        .write(expected_msg.clone())
        .await
        .unwrap();

    let sample = bundle.beta_reader.read_next_sample().await.unwrap();
    let received_msg = sample.take_data().unwrap();
    assert_eq!(received_msg, expected_msg);

    println!("BETA READER RECEIVE DATA FROM ALPHA WRITER");

    let sample = bundle.gamma_reader.read_next_sample().await.unwrap();
    let received_msg = sample.take_data().unwrap();
    assert_eq!(received_msg, expected_msg);

    println!("GAMMA READER RECEIVE DATA FROM ALPHA WRITER");

    bundle
        .beta_writer
        .write(expected_msg.clone())
        .await
        .unwrap();

    let sample = bundle.alpha_reader.read_next_sample().await.unwrap();
    let received_msg = sample.take_data().unwrap();
    assert_eq!(received_msg, expected_msg);

    println!("ALPHA READER RECEIVE DATA FROM BETA WRITER");

    let sample = bundle.gamma_reader.read_next_sample().await.unwrap();
    let received_msg = sample.take_data().unwrap();
    assert_eq!(received_msg, expected_msg);

    println!("GAMMA READER RECEIVE DATA FROM BETA WRITER");

    bundle
        .gamma_writer
        .write(expected_msg.clone())
        .await
        .unwrap();

    let sample = bundle.alpha_reader.read_next_sample().await.unwrap();
    let received_msg = sample.take_data().unwrap();
    assert_eq!(received_msg, expected_msg);

    println!("ALPHA READER RECEIVE DATA FROM GAMMA WRITER");

    let sample = bundle.beta_reader.read_next_sample().await.unwrap();
    let received_msg = sample.take_data().unwrap();
    assert_eq!(received_msg, expected_msg);

    println!("BETA READER RECEIVE DATA FROM GAMMA WRITER");
}
