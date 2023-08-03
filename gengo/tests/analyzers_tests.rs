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
