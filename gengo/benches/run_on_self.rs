use criterion::{criterion_group, criterion_main, Criterion};
use gengo::Builder;

fn head_benchmark(c: &mut Criterion) {
    for n in 0..3 {
        let rev = format!("HEAD{}", "^".repeat(n));
        let gengo = Builder::new(env!("CARGO_MANIFEST_DIR"), &rev)
            .build()
            .unwrap();
        c.bench_function(&format!("run on {}", rev), |b| {
            b.iter(|| gengo.analyze().unwrap())
        });
    }
}

criterion_group!(benches, head_benchmark);
criterion_main!(benches);
