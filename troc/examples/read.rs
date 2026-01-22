mod common;

use std::{sync::Arc, time::Instant};

use clap::Parser;
use common::{Message, OtlParam, set_up_log};
use troc::{DomainParticipantBuilder, HistoryQosPolicy, ReliabilityQosPolicy, TopicKind};

use tokio::{self, select};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

#[derive(Debug, Parser)]
struct CliArgs {
    #[arg(short = 'n', default_value_t = -1)]
    nb: i64,
    /// Number of Change to keep in history
    /// If value is 0, then it's a KeepAll policy
    #[arg(short = 'k', default_value_t = 0)]
    history_keep: usize,
    #[arg(short = 'b', long, default_value_t = false)]
    best_effort: bool,
    #[arg(long, env, default_value_t = false)]
    otl: bool,
}

#[tokio::main(flavor = "current_thread")]
#[instrument(name = "read_example")]
async fn main() {
    let mut cli_args = CliArgs::parse();

    set_up_log(cli_args.otl.then_some(OtlParam {
        service_name: "troc::examples::reader",
    }));

    let cancellation_token = Arc::new(CancellationToken::new());

    {
        let cancellation_token = cancellation_token.clone();
        ctrlc::set_handler(move || {
            cancellation_token.cancel();
        })
        .unwrap();
    }

    let mut domain_participant = DomainParticipantBuilder::new().with_domain(0).build().await;

    let qos = domain_participant
        .create_qos_builder()
        .reliability(if cli_args.best_effort {
            ReliabilityQosPolicy::BestEffort
        } else {
            ReliabilityQosPolicy::Reliable {
                max_blocking_time: Default::default(),
            }
        })
        .history(if let 1..=usize::MAX = cli_args.history_keep {
            HistoryQosPolicy::KeepLast {
                depth: cli_args.history_keep as u32,
            }
        } else {
            HistoryQosPolicy::KeepAll
        })
        .build();

    let topic =
        domain_participant.create_topic("/dds_example", "Message", &qos, TopicKind::WithKey);

    let mut subscriber = domain_participant.create_subscriber(&qos).await.unwrap();

    let mut data_reader = subscriber
        .create_datareader::<Message>(&topic, &qos)
        .await
        .unwrap();

    let mut start = Instant::now();

    loop {
        match cli_args.nb {
            i64::MIN..=-1 => {}
            0 => break,
            _ => cli_args.nb -= 1,
        }

        select! {
            _ = cancellation_token.cancelled() => {
                break
            }
            Ok(sample) = data_reader.read_next_sample() => {
                 if let Some(message) = sample.data() {
                    if message.id() == 0 {
                        start = Instant::now();
                    }

                    let elapsed = start.elapsed();

                    println!(
                        "Writer {} sent {} bytes at {}, id: {}, key: {}, elapsed time since msg_id 0: {} (us)",
                        message.writer_guid(),
                        message.payload_size(),
                        message.time(),
                        message.id(),
                        message.key(),
                        elapsed.as_micros()
                    );
                }
            }
            else => continue
        }
    }
}
