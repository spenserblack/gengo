use gengo::Analyzers;
use insta::assert_debug_snapshot;
use std::ffi::OsStr;

mod util;

#[test]
fn test_by_filepath_json_with_comments() {
    let fixture = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let analyzers = dbg!(Analyzers::from_yaml(fixture)).unwrap();
    let results = analyzers.by_filepath(OsStr::new("test.json"));
    assert_debug_snapshot!(results);
}

#[test]
fn test_by_shebang_shell() {
    let fixture = fixture_str!("test_check_shebang_shell-analyzers.yaml");
    let analyzers = dbg!(Analyzers::from_yaml(fixture)).unwrap();
    let results = analyzers.by_shebang(b"#!/bin/sh\necho hello");
    assert_debug_snapshot!(results);
}

#[test]
fn test_simple() {
    let fixture = fixture_str!("test_simple-analyzers.yaml");
    let contents = fixture_bytes!("test_simple-file.sh");
    let analyzers = dbg!(Analyzers::from_yaml(fixture)).unwrap();
    assert_debug_snapshot!("analyzers_tests__test_simple__by_shebang", analyzers.by_shebang(contents));
    assert_debug_snapshot!("analyzers_tests__test_simple__by_filepath", analyzers.by_filepath(OsStr::new("test.sh")));
    assert_debug_snapshot!("analyzers_tests__test_simple", analyzers.simple(OsStr::new("test.sh"), contents));
}
