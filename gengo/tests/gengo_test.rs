use gengo::Analyzers;
use gengo::Builder;
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

mod util;

#[test]
fn test_javascript() {
    let analyzers = fixture_str!("test_javascript-analyzers.yaml");
    let analyzers = Analyzers::from_yaml(analyzers).unwrap();
    let gengo = Builder::new(ROOT, "test/javascript")
        .analyzers(analyzers)
        .build()
        .unwrap();
    let results = gengo.analyze().unwrap();
    insta::assert_debug_snapshot!(results);
}
