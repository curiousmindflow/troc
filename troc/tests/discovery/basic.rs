//! This module regroup tests that assert the correct behavior of the basic discovery features
//!
//! Here we don't want to test:
//! - data exchange, we just want to assert a communication happened
//! - QoS combination, we just want the default QoS

use std::time::Duration;

use crate::{
    discovery::{DOMAIN_ID_98, DOMAIN_ID_99},
    fixture::{TwoParticipantsBundle, build_qos, setup_log, two_participants},
};
use troc::{DomainParticipantListener, QosPolicy, TopicKind};
use troc_core::DurationKind;

use rstest::*;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[tokio::test]
async fn participant_must_discover(
    #[from(setup_log)] _setup_log: (),
    #[from(two_participants)]
    #[with("discovery/basic/discovery_by_listener")]
    #[future]
    two_participants: TwoParticipantsBundle,
) {
    let bundle = two_participants.await;
    participant_discovery(&bundle).await;
}

#[rstest]
#[timeout(Duration::from_secs(10))]
#[tokio::test]
async fn participant_stale_detection(
    #[from(setup_log)] _setup_log: (),
    #[with("discovery/basic/stale_detection")]
    #[future]
    two_participants: TwoParticipantsBundle,
) {
    let bundle = two_participants.await;
    let (mut alpha_participant_listener, _) = participant_discovery(&bundle).await;

    let TwoParticipantsBundle {
        alpha_domain_participant: _,
        beta_domain_participant,
        alpha_publisher,
        beta_subscriber,
        alpha_writer,
        beta_reader,
    } = bundle;

    let beta_domain_participant_guid = beta_domain_participant.get_guid();

    drop(beta_reader);
    drop(beta_subscriber);
    drop(beta_domain_participant);

    let _alpha_publisher = alpha_publisher;
    let _alpha_writer = alpha_writer;

    let event = alpha_participant_listener
        .wait_participant_removed(DurationKind::Infinite)
        .await
        .unwrap();
    assert_eq!(event.get_guid(), beta_domain_participant_guid);
}

#[rstest]
#[tokio::test]
async fn domain_isolation(
    #[from(setup_log)] _setup_log: (),
    #[from(build_qos)] _qos: QosPolicy,
    #[with(
        "discovery/basic/domain_isolation",
        TopicKind::NoKey,
        _qos,
        _qos,
        DOMAIN_ID_98,
        DOMAIN_ID_99
    )]
    #[future]
    two_participants: TwoParticipantsBundle,
) {
    let bundle = two_participants.await;
    let result = tokio::time::timeout(Duration::from_secs(5), participant_discovery(&bundle)).await;
    assert!(result.is_err());
}

async fn participant_discovery(
    bundle: &TwoParticipantsBundle,
) -> (DomainParticipantListener, DomainParticipantListener) {
    let mut alpha_listener = bundle
        .alpha_domain_participant
        .get_listener()
        .await
        .unwrap();
    let mut beta_listener = bundle.beta_domain_participant.get_listener().await.unwrap();

    let event = alpha_listener
        .wait_participant_discovered(DurationKind::Infinite)
        .await
        .unwrap();

    assert_eq!(event.get_guid(), bundle.beta_domain_participant.get_guid());

    let event = beta_listener
        .wait_participant_discovered(DurationKind::Infinite)
        .await
        .unwrap();

    assert_eq!(event.get_guid(), bundle.alpha_domain_participant.get_guid());

    (alpha_listener, beta_listener)
}
