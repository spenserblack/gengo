use criterion::{Criterion, criterion_group, criterion_main};
use gengo::{Builder, Git};

fn git_benchmark(c: &mut Criterion) {
    let revs = {
        let mut revs: Vec<_> = (0..3).map(|n| format!("HEAD{}", "^".repeat(n))).collect();
        revs.extend_from_slice(&["test/javascript".into()]);
        revs
    };
    for rev in revs {
        let git = Git::new(env!("CARGO_MANIFEST_DIR"), &rev).unwrap();
        let gengo = Builder::new(git).build().unwrap();
        c.bench_function(&format!("run on {}", rev), |b| {
            b.iter(|| gengo.analyze().unwrap())
        });
    }
}

criterion_group!(benches, git_benchmark);
criterion_main!(benches);
