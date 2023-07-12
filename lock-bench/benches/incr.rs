use std::iter::successors;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lock_bench::{incr, incr_locked};

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("incr");
    for i in successors(Some(2usize), |prev| Some(prev << 1)).take(16) {
        group.throughput(Throughput::Elements(i as _));

        group.bench_with_input(BenchmarkId::new("no-lock", i), &i, |b, count| {
            b.iter(|| incr(10, *count))
        });

        group.bench_with_input(BenchmarkId::new("locked", i), &i, |b, count| {
            b.iter(|| incr_locked(10, *count))
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
