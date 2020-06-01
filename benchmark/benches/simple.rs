use criterion::{criterion_group, criterion_main, Criterion, black_box};
use simple::{Q, run, parse, serialize, GQLResponse};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("run", |b| b.iter(|| run(black_box(Q))));
    c.bench_function("parse", |b| b.iter(|| parse(black_box(Q))));
    let res = GQLResponse(run(Q));
    c.bench_function("serialize", |b| b.iter(|| serialize(black_box(&res))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);