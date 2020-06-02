use chat::{Q, S};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graphql_benchmark::{parse, run, serialize, GQLResponse};

pub fn bench(c: &mut Criterion) {
    c.bench_function("chat run", |b| b.iter(|| run(&S, black_box(Q))));
    c.bench_function("chat parse", |b| b.iter(|| parse(black_box(Q))));
    let res = GQLResponse(Ok(run(&S, Q)));
    c.bench_function("chat serialize", |b| b.iter(|| serialize(black_box(&res))));
}

criterion_group!(chat, bench);
criterion_main!(chat);
