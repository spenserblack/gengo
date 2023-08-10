use gengo::Analyzers;
use insta::assert_debug_snapshot;
use std::ffi::OsStr;

mod util;

#[test]
fn test_by_filepath_json_with_comments() {
    let fixture = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let analyzers = Analyzers::from_yaml(fixture).unwrap();
    let results = analyzers.by_filepath(OsStr::new("test.json"));
    assert_debug_snapshot!(results);
}

#[test]
fn test_by_shebang_shell() {
    let fixture = fixture_str!("test_check_shebang_shell-analyzers.yaml");
    let analyzers = Analyzers::from_yaml(fixture).unwrap();
    let results = analyzers.by_shebang(b"#!/bin/sh\necho hello");
    assert_debug_snapshot!(results);
}

#[test]
fn test_simple() {
    let fixture = fixture_str!("test_simple-analyzers.yaml");
    let contents = fixture_bytes!("test_simple-file.sh");
    let analyzers = Analyzers::from_yaml(fixture).unwrap();
    assert_debug_snapshot!(
        "analyzers_tests__test_simple__by_shebang",
        analyzers.by_shebang(contents)
    );
    assert_debug_snapshot!(
        "analyzers_tests__test_simple__by_filepath",
        analyzers.by_filepath(OsStr::new("test.sh"))
    );
    assert_debug_snapshot!(
        "analyzers_tests__test_simple",
        analyzers.simple(OsStr::new("test.sh"), contents)
    );
}

#[test]
fn test_with_heuristics() {
    let fixture = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let filepath = OsStr::new("test.json");
    let contents = fixture_bytes!("test_check_json_with_comments-file.json");
    let analyzers = Analyzers::from_yaml(fixture).unwrap();
    assert_debug_snapshot!(
        "analyzers_tests__test_with_heuristics__simple",
        analyzers.simple(filepath, contents)
    );
    assert_debug_snapshot!(
        "analyzers_tests__test_with_heuristics__with_heuristics",
        analyzers.with_heuristics(filepath, contents, 1 << 20)
    );
}

#[test]
fn test_no_heuristic_match_returns_original_set() {
    let fixture = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let filepath = OsStr::new("test.json");
    let contents = br#"{"type": "maybe not JSON with comments"}"#;
    let analyzers = Analyzers::from_yaml(fixture).unwrap();
    let result = analyzers.with_heuristics(filepath, contents, 1 << 20);
    assert_eq!(
        result.len(),
        2,
        "Both JSON and JSON with Comments be returned"
    );
}

#[test]
fn test_pick_find_one() {
    let fixture = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let filepath = OsStr::new("test.json");
    let contents = fixture_bytes!("test_check_json_with_comments-file.json");
    let analyzers = Analyzers::from_yaml(fixture).unwrap();
    let language = analyzers.pick(filepath, contents, 1 << 20).unwrap();
    assert_eq!(language.name(), "JSON with Comments");
}

#[test]
fn test_pick_find_none() {
    let fixture = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let filepath = OsStr::new("test.yaml");
    let contents = b"---\nfoo: bar\n";
    let analyzers = Analyzers::from_yaml(fixture).unwrap();
    let language = analyzers.pick(filepath, contents, 1);
    assert!(language.is_none());
}

#[test]
fn test_pick_find_multiple() {
    let fixture = fixture_str!("test_check_json_with_comments-analyzers.yaml");
    let filepath = OsStr::new("test.json");
    let contents = br#"{"msg": "Comments? IDK"}"#;
    let analyzers = Analyzers::from_yaml(fixture).unwrap();
    let language = analyzers.pick(filepath, contents, 1 << 20).unwrap();
    assert_eq!(
        language.name(),
        "JSON",
        "It should pick the language with the higher priority"
    );
}
