use gengo::Builder;
use gengo::Analyzers;
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

mod util;

#[test]
fn test_javascript() {
    let analyzers = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let analyzers = Analyzers::from_yaml(analyzers).unwrap();
    let gengo = Builder::new(ROOT)
        .analyzers(analyzers)
        .build()
        .unwrap();
    let results = gengo.analyze("test/javascript").unwrap();
    insta::assert_debug_snapshot!(results);
}
