use gengo::Analyzers;
use gengo::Builder;
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

mod util;

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn test_javascript() {
    let analyzers = fixture_str!("test_javascript-analyzers.yaml");
    let analyzers = Analyzers::from_yaml(analyzers).unwrap();
    let gengo = Builder::new(ROOT).analyzers(analyzers).build().unwrap();
    let results = gengo.analyze("test/javascript").unwrap();
    insta::assert_debug_snapshot!(results);
}

// TODO Add test_javascript_windows
