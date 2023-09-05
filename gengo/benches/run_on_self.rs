use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gengo::Builder;

fn head_benchmark(c: &mut Criterion) {
    let mut gengo = Builder::new(env!("CARGO_MANIFEST_DIR")).build().unwrap();
    for n in 0..3 {
        let rev = format!("HEAD{}", "^".repeat(n));
        c.bench_function(&format!("run on {}", rev), |b| {
            b.iter(|| gengo.analyze(black_box(&rev)).unwrap())
        });
    }
}

criterion_group!(benches, head_benchmark);
criterion_main!(benches);
