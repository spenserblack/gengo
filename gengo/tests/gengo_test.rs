use gengo::{Builder, Git};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn test_git_javascript() {
    // TODO It's not great to use a snapshot test on a type that doesn't
    // guarantee order. Improve this test.
    let git = Git::new(ROOT, "test/javascript").unwrap();
    let gengo = Builder::new(git).build().unwrap();
    let analysis = gengo.analyze().unwrap();
    let mut results: Vec<_> = analysis.iter().collect();
    results.sort_by_key(|(path, _)| path.to_owned());
    insta::assert_debug_snapshot!(results);
}
