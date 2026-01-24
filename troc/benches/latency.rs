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
    targets = one_to_one
}

criterion_main!(bench);

pub fn one_to_one(c: &mut Criterion) {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    let mut group = c.benchmark_group("one_to_one");
    group.warm_up_time(Duration::from_secs(2));

    for size in [1, 4 * K, 32 * K, 59 * K].into_iter() {
        let bundle = runtime.block_on(SimpleDDSBundle::new(
            TopicKind::NoKey,
            ReliabilityKind::BestEffort,
            "/benchmark/latency/besteffort_nokey",
            HistoryQosPolicy::KeepLast { depth: 1 },
        ));
        let bundle = Arc::new(Mutex::new(bundle));

        group.throughput(criterion::Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("one_to_one", size), &size, |b, size| {
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
        });
    }
    group.finish()
}

pub fn one_to_many(c: &mut Criterion) {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    let mut group = c.benchmark_group("one_to_many");
    group.warm_up_time(Duration::from_secs(2));

    let bundle = runtime.block_on(OneWriterManyReaderDDSBundle::new(
        TopicKind::NoKey,
        ReliabilityKind::BestEffort,
        "/benchmark/latency/one_writer_two_readers",
        HistoryQosPolicy::KeepLast { depth: 1 },
    ));
    let bundle = Arc::new(Mutex::new(bundle));

    for size in [1, 4 * K, 32 * K, 59 * K].into_iter() {
        group.throughput(criterion::Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("one_to_many", size), &size, |b, size| {
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
        });
    }
    group.finish()
}
