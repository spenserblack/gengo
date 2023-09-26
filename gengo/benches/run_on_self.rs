use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gengo::Builder;

fn head_benchmark(c: &mut Criterion) {
    let gengo = Builder::new(env!("CARGO_MANIFEST_DIR")).build().unwrap();
    let revs = {
        let mut revs: Vec<_> = (0..3).map(|n| format!("HEAD{}", "^".repeat(n))).collect();
        revs.extend_from_slice(&["test/javascript".into()]);
        revs
    };
    for rev in revs {
        c.bench_function(&format!("run on {}", rev), |b| {
            b.iter(|| gengo.analyze(black_box(&rev)).unwrap())
        });
    }
}

criterion_group!(benches, head_benchmark);
criterion_main!(benches);
