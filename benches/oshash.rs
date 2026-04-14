use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;

const TEST_FILE: &str = "test-resources/testdata";

fn bench_hash_file(c: &mut Criterion) {
    c.bench_function("oshash_file", |b| {
        b.iter(|| {
            oshash::oshash(TEST_FILE).unwrap();
        })
    });
}

fn bench_hash_buf(c: &mut Criterion) {
    c.bench_function("oshash_buf", |b| {
        b.iter_batched(
            || {
                let file = File::open(TEST_FILE).unwrap();
                let len = file.metadata().unwrap().len();
                (file, len)
            },
            |(mut file, len)| {
                oshash::oshash_buf(&mut file, len).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_hash_file, bench_hash_buf);
criterion_main!(benches);
