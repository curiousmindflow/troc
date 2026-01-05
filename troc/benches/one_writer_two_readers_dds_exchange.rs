mod common;

use crate::common::resources::{
    BenchMessage, OneWriterManyReaderDDSBundle, one_writer_two_readers_dds_exchange,
};
use tokio::time::{Duration, sleep};
use troc_core::{HistoryQosPolicy, ReliabilityKind, TopicKind};

#[tokio::main]
async fn main() {
    let mut bundle = OneWriterManyReaderDDSBundle::new(
        TopicKind::NoKey,
        ReliabilityKind::BestEffort,
        "/benchmark/test/dds_exchange/throughput/besteffort_nokey",
        HistoryQosPolicy::KeepLast { depth: 1 },
    )
    .await;

    sleep(Duration::from_secs(2)).await;

    let msg = BenchMessage::new(59 * 1024);

    for _ in 0..100 {
        let msg = msg.clone();

        tokio::time::timeout(
            Duration::from_millis(100),
            one_writer_two_readers_dds_exchange(&mut bundle, msg),
        )
        .await
        .unwrap();
    }
}
