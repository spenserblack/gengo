use gengo::{Analyzers, Builder, Directory, Git};
use std::path::Path;
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

mod util;

#[test]
fn test_git_javascript() {
    // TODO It's not great to use a snapshot test on a type that doesn't
    // guarantee order. Improve this test.
    let analyzers = fixture_str!("test_javascript-analyzers.yaml");
    let analyzers = Analyzers::from_yaml(analyzers).unwrap();
    let git = Git::new(ROOT, "test/javascript").unwrap();
    let gengo = Builder::new(git).analyzers(analyzers).build().unwrap();
    let analysis = gengo.analyze().unwrap();
    let mut results: Vec<_> = analysis.iter().collect();
    results.sort_by_key(|(path, _)| path.to_owned());
    insta::assert_debug_snapshot!(results);
}

#[test]
#[cfg_attr(windows, ignore)]
fn test_directory_fixtures() {
    let directory = Path::new(ROOT).join("tests/fixtures");
    let directory = Directory::new(directory, 1 << 20).unwrap();
    let gengo = Builder::new(directory).build().unwrap();
    let analysis = gengo.analyze().unwrap();
    // NOTE Stripping prefix for more consistent snapshots.
    let mut results: Vec<_> = analysis
        .iter()
        .map(|(path, entry)| (path.strip_prefix(ROOT).unwrap(), entry))
        .collect();
    results.sort_by_key(|(path, _)| path.to_owned());
    insta::assert_debug_snapshot!(results);
}
