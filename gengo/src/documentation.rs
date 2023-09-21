use super::GLOB_MATCH_OPTIONS;
use glob::Pattern;
use std::path::Path;

pub struct Documentation {
    globs: Vec<Pattern>,
}

impl Documentation {
    pub fn new() -> Self {
        let globs = Self::globs();

        Self { globs }
    }

    pub fn is_documentation<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.is_documentation_no_read(&filepath)
            || self.is_documentation_with_read(&filepath, contents)
    }

    fn is_documentation_no_read<P: AsRef<Path>>(&self, filepath: P) -> bool {
        self.globs
            .iter()
            .any(|g| g.matches_path_with(filepath.as_ref(), GLOB_MATCH_OPTIONS))
    }

    fn is_documentation_with_read<P: AsRef<Path>>(&self, _filepath: P, _contents: &[u8]) -> bool {
        false
    }

    fn globs() -> Vec<Pattern> {
        [
            // Directories
            "**/docs/**",
            // Files
            "**/CHANGELOG",
            "**/CHANGELOG.*",
            "**/HACKING",
            "**/HACKING.*",
            "**/README",
            "**/README.*",
        ]
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
        case("docs/something.md", true),
        case("src/something.rs", false),
        case("docs/subfolder/something.md", true),
        case("", false),
        case("docs", false),
        case("CHANGELOG", true),
        case("CHANGELOG.txt", true),
        case("CHANGELOG.md", true),
        case("HACKING", true),
        case("HACKING.txt", true),
        case("HACKING.md", true),
        case("README", true),
        case("README.txt", true),
        case("README.md", true)
    )]
    fn test_is_documentation_no_read(filepath: &str, expected: bool) {
        let documentation = Documentation::new();
        assert_eq!(documentation.is_documentation_no_read(filepath), expected);
    }
}
