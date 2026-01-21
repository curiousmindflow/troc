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
        alpha_domain_participant,
        beta_domain_participant: _,
        beta_publisher,
        alpha_subscriber,
        beta_writer,
        alpha_reader,
    } = bundle;

    let beta_domain_participant_guid = alpha_domain_participant.get_guid();

    drop(alpha_reader);
    drop(alpha_subscriber);
    drop(alpha_domain_participant);

    let _beta_publisher = beta_publisher;
    let _beta_writer = beta_writer;

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

    assert_eq!(
        event.get_guid().get_guid_prefix(),
        bundle.beta_domain_participant.get_guid().get_guid_prefix()
    );

    let event = beta_listener
        .wait_participant_discovered(DurationKind::Infinite)
        .await
        .unwrap();

    assert_eq!(
        event.get_guid().get_guid_prefix(),
        bundle.alpha_domain_participant.get_guid().get_guid_prefix()
    );

    (alpha_listener, beta_listener)
}
