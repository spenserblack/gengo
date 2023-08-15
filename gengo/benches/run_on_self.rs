use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gengo::Builder;

fn criterion_benchmark(c: &mut Criterion) {
    let gengo = Builder::new(env!("CARGO_MANIFEST_DIR")).build().unwrap();
    c.bench_function("run on self", |b| {
        b.iter(|| gengo.analyze(black_box("HEAD")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
