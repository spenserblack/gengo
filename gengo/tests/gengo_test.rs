use gengo::Analyzers;
use gengo::Builder;
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

mod util;

#[test]
#[ignore = "HashMaps have an unpredictable order"]
fn test_javascript() {
    // TODO It's not great to use a snapshot test on a type that doesn't
    // guarantee order. Improve this test.
    let analyzers = fixture_str!("test_javascript-analyzers.yaml");
    let analyzers = Analyzers::from_yaml(analyzers).unwrap();
    let gengo = Builder::new(ROOT, "test/javascript")
        .analyzers(analyzers)
        .build()
        .unwrap();
    let results = gengo.analyze().unwrap();
    insta::assert_debug_snapshot!(results);
}
