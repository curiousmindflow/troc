use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use common::resources::{
    BenchMessage, K, OneWriterManyReaderDDSBundle, SimpleDDSBundle,
    one_writer_one_reader_dds_exchange, one_writer_two_readers_dds_exchange,
};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use pprof::criterion::{Output, PProfProfiler};
use tokio::{runtime::Builder, sync::Mutex};
use troc_core::TopicKind;
use troc_core::{HistoryQosPolicy, ReliabilityKind};

mod common;

criterion_group! {
    name = bench;
    config = Criterion::default()
        .with_profiler(
            PProfProfiler::new(100, Output::Flamegraph(None))
        );
    targets = throughput
}

criterion_main!(bench);

pub fn throughput(c: &mut Criterion) {
    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

    let mut group = c.benchmark_group("throughput");
    group.warm_up_time(Duration::from_secs(2));

    for size in [1, 59 * K, 128 * K].into_iter() {
        group.throughput(criterion::Throughput::Bytes(size as u64));

        let bundle = runtime.block_on(SimpleDDSBundle::new(
            TopicKind::NoKey,
            ReliabilityKind::BestEffort,
            "/benchmark/throughput/besteffort_nokey",
            HistoryQosPolicy::KeepLast { depth: 1 },
        ));
        let bundle = Arc::new(Mutex::new(bundle));

        group.bench_with_input(
            BenchmarkId::new("one_writer_one_reader", size),
            &size,
            |b, size| {
                let bundle = bundle.clone();

                b.to_async(&runtime).iter_custom(move |mut iters| {
                    let bundle = bundle.clone();

                    async move {
                        let mut total_elapsed = Duration::default();
                        let mut bundle = bundle.lock().await;

                        while iters > 0 {
                            let msg = BenchMessage::new(*size);
                            let start = Instant::now();

                            if tokio::time::timeout(
                                Duration::from_millis(200),
                                black_box(one_writer_one_reader_dds_exchange(&mut bundle, msg)),
                            )
                            .await
                            .is_ok()
                            {
                                total_elapsed += start.elapsed();

                                iters -= 1;
                            } else {
                                eprintln!("exchange didn't take place");
                            }
                        }

                        total_elapsed
                    }
                });
            },
        );

        let bundle = runtime.block_on(OneWriterManyReaderDDSBundle::new(
            TopicKind::NoKey,
            ReliabilityKind::BestEffort,
            "/benchmark/throughput/one_writer_two_readers",
            HistoryQosPolicy::KeepLast { depth: 1 },
        ));
        let bundle = Arc::new(Mutex::new(bundle));

        group.bench_with_input(
            BenchmarkId::new("one_writer_two_readers", size),
            &size,
            |b, size| {
                let bundle = bundle.clone();

                b.to_async(&runtime).iter_custom(move |mut iters| {
                    let bundle = bundle.clone();

                    async move {
                        let mut total_elapsed = Duration::default();
                        let mut bundle = bundle.lock().await;

                        while iters > 0 {
                            let msg = BenchMessage::new(*size);
                            let start = Instant::now();

                            if tokio::time::timeout(
                                Duration::from_millis(200),
                                black_box(one_writer_two_readers_dds_exchange(&mut bundle, msg)),
                            )
                            .await
                            .is_ok()
                            {
                                total_elapsed += start.elapsed();

                                iters -= 1;
                            } else {
                                eprintln!("exchange didn't take place");
                            }
                        }

                        total_elapsed
                    }
                });
            },
        );
    }
    group.finish()
}
