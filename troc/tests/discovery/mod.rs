use std::time::Duration;

use crate::fixture::{DummyStruct, TwoParticipantsBundle};

mod basic;
// mod complex;
// mod matching;

const DOMAIN_ID_98: u32 = 98;
const DOMAIN_ID_99: u32 = 99;

pub async fn exchange(
    mut bundle: TwoParticipantsBundle,
    timeout_delay_seconds: u64,
) -> Result<(), anyhow::Error> {
    let expected_msg = DummyStruct::new(0, &[]);

    bundle
        .alpha_writer
        .write(expected_msg.clone())
        .await
        .unwrap();
    let sample = bundle
        .beta_reader
        .read_next_sample_timeout(Duration::from_secs(timeout_delay_seconds))
        .await?;
    let received_msg = sample.take_data().unwrap();

    assert_eq!(received_msg, expected_msg);

    Ok(())
}
