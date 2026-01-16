use std::time::Duration;

use bytes::BytesMut;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pprof::criterion::{Output, PProfProfiler};
use troc_core::{ContentNature, K, Message, MessageFactory, SerializedData};

criterion_main!(main_bench);

criterion_group! {
    name = main_bench;
    config = Criterion::default()
        .with_profiler(
            PProfProfiler::new(100, Output::Flamegraph(None))
        );
    targets = serialize, deserialize
}

pub fn serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");

    for size in [512, 4 * K, 8 * K, 16 * K, 32 * K, 59 * K].into_iter() {
        group.throughput(criterion::Throughput::Bytes(size as u64));
        group.warm_up_time(Duration::from_secs(5));

        let msg = MessageFactory::new(Default::default())
            .message()
            .reader(Default::default())
            .writer(Default::default())
            .data(
                ContentNature::Data,
                Default::default(),
                None,
                Some(SerializedData::from_slice(
                    &std::iter::repeat_n(7u8, size as usize).collect::<Vec<u8>>(),
                )),
            )
            .build();

        let mut buffer = BytesMut::zeroed(64 * 1024);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _size| {
            b.iter(|| {
                msg.serialize_to(&mut buffer).unwrap();
            });
        });
    }
}

pub fn deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize");

    for size in [512, 4 * K, 8 * K, 16 * K, 32 * K, 59 * K].into_iter() {
        group.throughput(criterion::Throughput::Bytes(size as u64));
        group.warm_up_time(Duration::from_secs(5));

        let msg = MessageFactory::new(Default::default())
            .message()
            .reader(Default::default())
            .writer(Default::default())
            .data(
                ContentNature::Data,
                Default::default(),
                None,
                Some(SerializedData::from_slice(
                    &std::iter::repeat_n(7u8, size as usize).collect::<Vec<u8>>(),
                )),
            )
            .build();

        let mut buffer = BytesMut::zeroed(64 * 1024);

        let size = msg.serialize_to(&mut buffer).unwrap();
        let intermediate_buffer = buffer.split_to(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _size| {
            b.iter(|| {
                Message::deserialize_from(&intermediate_buffer).unwrap();
            });
        });
    }
}
