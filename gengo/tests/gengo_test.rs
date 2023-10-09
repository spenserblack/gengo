use gengo::{Analyzers, Builder, Git};
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

mod util;

#[test]
fn test_javascript() {
    // TODO It's not great to use a snapshot test on a type that doesn't
    // guarantee order. Improve this test.
    let analyzers = fixture_str!("test_javascript-analyzers.yaml");
    let analyzers = Analyzers::from_yaml(analyzers).unwrap();
    let git = Git::new(ROOT, "test/javascript").unwrap();
    let gengo = Builder::new(git).analyzers(analyzers).build().unwrap();
    let analysis = gengo.analyze().unwrap();
    let mut results: Vec<_> = analysis.iter().collect();
    results.sort_by_key(|(path, _)| path);
    insta::assert_debug_snapshot!(results);
}
