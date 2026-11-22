//! Benchmarks for the async API (requires the `tokio` feature).
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use tokio::fs::File;
use tokio::runtime::{Builder, Runtime};

const TEST_FILE: &str = "test-resources/testdata";

fn runtime() -> Runtime {
    Builder::new_multi_thread().enable_all().build().unwrap()
}

fn bench_hash_file_async(c: &mut Criterion) {
    let rt = runtime();
    c.bench_function("oshash_async", |b| {
        b.to_async(&rt).iter(|| async {
            oshash::oshash_async(TEST_FILE).await.unwrap();
        })
    });
}

fn bench_hash_buf_async(c: &mut Criterion) {
    let rt = runtime();
    c.bench_function("oshash_buf_async", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let file = std::fs::File::open(TEST_FILE).unwrap();
                let len = file.metadata().unwrap().len();
                (File::from_std(file), len)
            },
            |(mut file, len)| async move {
                oshash::oshash_buf_async(&mut file, len).await.unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_hash_file_async, bench_hash_buf_async);
criterion_main!(benches);
