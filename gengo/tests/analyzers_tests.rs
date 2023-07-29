use insta::assert_debug_snapshot;
use gengo::Analyzers;
use std::ffi::OsStr;

mod util;

#[test]
fn test_check_json_with_comments() {
    let fixture = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let analyzers = dbg!(Analyzers::from_yaml(fixture));
    let data = fixture_bytes!("test_check_json_with_comments-file.json");
    let results = analyzers.check(OsStr::new("test.json"), data);
    assert_debug_snapshot!(results);
}
