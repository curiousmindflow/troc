mod common;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use clap::Parser;
use common::{OtlParam, Shape, parse_size, set_up_log};
use troc::{DomainParticipantBuilder, HistoryQosPolicy, ReliabilityQosPolicy, TopicKind};

use tokio::time::sleep;
use tracing::instrument;

use crate::common::Message;

#[derive(Debug, Parser)]
struct CliArgs {
    /// The data's size to be sent in Bytes
    /// Must be in the form <nb of bytes>[<unit>]
    /// The unit can be:
    /// - o | O: Bytes
    /// - k | K: Kilobyte
    /// - m | M: Megabyte
    /// - g | G: Gigabyte
    ///     When <unit> is not specified, the default value is Bytes
    #[arg(short = 'p', value_parser = parse_size, default_value_t = 1, verbatim_doc_comment)]
    size: u64,
    /// Delay between sending, in us
    #[arg(short = 'd', default_value_t = 1000000)]
    send_delay: u64,
    /// Number of time the data must be sent
    /// If the value is negative, it will send indefinitely
    #[arg(short, default_value_t = -1)]
    nb: i64,
    /// Period of time that must be waited before sending the first message, in ms
    #[arg(short = 's', default_value_t = 0)]
    setup_delay: u64,
    /// Period of time the process must be kept alive (it wait in an infinite loop of 250ms delay/iter)
    /// once all data has been sent (ms).
    /// If the value is -1, then the process will wait forever
    #[arg(short = 'a', default_value_t = -1)]
    keep_alive_delay: i64,
    /// Number of Change to keep in history
    /// If value is 0, then it's a KeepAll policy
    #[arg(short = 'k', default_value_t = 0)]
    history_keep: usize,
    #[arg(short = 'b', long, default_value_t = false)]
    best_effort: bool,
    #[arg(long, env, default_value_t = false)]
    otl: bool,
}

// #[tokio::main(flavor = "multi_thread", worker_threads = 8)]
#[tokio::main(flavor = "current_thread")]
#[instrument(level = "TRACE", name = "write_example")]
async fn main() {
    let cli_args = CliArgs::parse();

    set_up_log(cli_args.otl.then_some(OtlParam {
        service_name: "troc::examples::write",
    }));

    let running_0 = Arc::new(AtomicBool::new(true));
    let r = running_0.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst)).unwrap();

    let mut domain_participant = DomainParticipantBuilder::new().with_domain(0).build();
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

    let mut publisher = domain_participant.create_publisher(&qos).await.unwrap();

    let mut data_writer = publisher
        .create_datawriter::<Message>(&topic, &qos)
        .await
        .unwrap();

    sleep(Duration::from_millis(cli_args.setup_delay)).await;

    let payload = std::iter::repeat_n(0x81, cli_args.size as usize).collect::<Vec<u8>>();

    let mut i = cli_args.nb;
    let mut toggle = true;
    let mut id = 0;

    while running_0.load(Ordering::SeqCst) {
        match i {
            i64::MIN..=-1 => {}
            0 => break,
            _ => i -= 1,
        }

        toggle = !toggle;

        let message = Message::new(data_writer.get_guid(), &payload, id, toggle as u64);
        // let message = Shape {
        //     color: "RED".to_string(),
        //     x: 20,
        //     y: 20,
        //     shapesize: 100,
        // };

        data_writer.write(message.clone()).await.unwrap();

        println!(
            "Writer {} sent {} bytes at {}, id: {}, key: {}",
            message.writer_guid(),
            message.payload_size(),
            message.time(),
            message.id(),
            message.key()
        );

        if cli_args.send_delay != 0 {
            sleep(Duration::from_micros(cli_args.send_delay)).await;
        }

        id += 1;
    }

    if cli_args.keep_alive_delay < 0 {
        while running_0.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(250)).await;
        }
    } else {
        sleep(Duration::from_millis(cli_args.keep_alive_delay as u64)).await;
    }
}
