use std::time::Duration;

use troc::{DdsError, TopicKind};

use rstest::*;

use crate::fixture::{
    DummyStruct, TwoParticipantsBundle, build_payload, setup_log, two_participants,
};

#[rstest]
#[timeout(Duration::from_secs(10))]
#[tokio::test]
async fn endpoints_have_matched(
    #[from(setup_log)] _setup_log: (),
    #[with(format!("comm/keyed/exchange"), TopicKind::WithKey)]
    #[future]
    two_participants: TwoParticipantsBundle,
    #[from(build_payload)] payload: Vec<u8>,
) {
    let mut bundle = two_participants.await;

    let mut msg = DummyStruct::new(1, &payload);

    bundle.beta_writer.write(msg.clone()).await.unwrap();

    msg.id = 0;
    match bundle
        .alpha_reader
        .read_next_sample_instance_timeout(&msg, Duration::from_secs(5))
        .await
    {
        Err(DdsError::Timeout { .. }) => (),
        _ => panic!(),
    }

    msg.id = 1;
    bundle.beta_writer.write(msg.clone()).await.unwrap();
    bundle
        .alpha_reader
        .read_next_sample_instance(&msg)
        .await
        .unwrap();
}
