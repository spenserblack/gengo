use super::GLOB_MATCH_OPTIONS;
use glob::Pattern;
use std::path::Path;

pub struct Vendored {
    globs: Vec<Pattern>,
}

impl Vendored {
    pub fn new() -> Self {
        let globs = Self::globs();

        Self { globs }
    }

    pub fn is_vendored<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.is_vendored_no_read(&filepath) || self.is_vendored_with_read(&filepath, contents)
    }

    fn is_vendored_no_read<P: AsRef<Path>>(&self, filepath: P) -> bool {
        self.globs
            .iter()
            .any(|g| g.matches_path_with(filepath.as_ref(), GLOB_MATCH_OPTIONS))
    }

    fn is_vendored_with_read<P: AsRef<Path>>(&self, _filepath: P, _contents: &[u8]) -> bool {
        false
    }

    fn globs() -> Vec<Pattern> {
        ["**/node_modules/**", "**/tests/fixtures/**"]
            .into_iter()
            .map(|g| Pattern::new(g).unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        filepath,
        expected,
        case("node_modules/library/something.js", true),
        case("src/something.rs", false),
        case("node_modules/subfolder/something.js", true),
        case("", false),
        case("node_modules", false),
        case("tests/fixtures/foo.json", true),
        case("package/tests/fixtures/foo.json", true)
    )]
    fn test_is_vendored_no_read(filepath: &str, expected: bool) {
        let vendored = Vendored::new();
        assert_eq!(vendored.is_vendored_no_read(filepath), expected);
    }
}
