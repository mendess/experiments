use criterion::{criterion_group, criterion_main, Criterion};

pub fn bench<const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("available_memory_{N}"));
    group.bench_function("readln", |b| {
        b.iter(|| available_memory::ram_readln::<N>().unwrap())
    });
    group.bench_function("streamparser", |b| {
        b.iter(|| available_memory::ram_streaming::<N>().unwrap())
    });
}

criterion_group!(
    benches,
    bench::<128>,
    bench::<256>,
    bench::<512>,
    bench::<1024>,
    bench::<4096>,
);
criterion_main!(benches);
