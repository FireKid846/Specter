use criterion::{criterion_group, criterion_main, Criterion};
// Search benchmarks will be added as search is implemented.
fn bench_placeholder(c: &mut Criterion) {
    c.bench_function("placeholder", |b| b.iter(|| 42u64));
}
criterion_group!(benches, bench_placeholder);
criterion_main!(benches);
