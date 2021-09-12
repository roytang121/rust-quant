use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use std::error::Error;

mod dataset;

fn encode_json_simple(value: &serde_json::Value) -> Result<String, Box<dyn Error>> {
    let result = serde_json::to_string(&value)?;
    Ok(result)
}

fn parse_json_simple(value: &mut str) -> Result<serde_json::Value, Box<dyn Error>> {
    let result = serde_json::from_str(value)?;
    Ok(result)
}

fn criterion_benchmark(c: &mut Criterion) {
    let simple_dataset = dataset::simple_dataset();
    let bytes = simple_dataset.as_bytes();
    let json = serde_json::from_str(simple_dataset.as_str()).unwrap();
    let mut group = c.benchmark_group("encode_json_simple");
    group
        .sample_size(500)
        // .warm_up_time(Duration::from_secs(3))
        .throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("encode_json", |b| b.iter(|| encode_json_simple(&json)));
    group.finish();
}

fn criterion_benchmark_4(c: &mut Criterion) {
    let mut twitter_dataset = dataset::twitter_dataset();
    let bytes = twitter_dataset.as_bytes();
    let mut group = c.benchmark_group("parse_json_simple");
    group
        .sample_size(500)
        // .warm_up_time(Duration::from_secs(3))
        .throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("parse_json", |b| {
        b.iter(|| parse_json_simple(twitter_dataset.as_mut_str()))
    });
    group.finish();
}

criterion_group!(
    benches,
    criterion_benchmark,
    criterion_benchmark_2,
    criterion_benchmark_3,
    criterion_benchmark_4
);
criterion_main!(benches);
