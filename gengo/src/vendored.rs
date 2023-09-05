use super::GLOB_MATCH_OPTIONS;
use glob::Pattern;
use std::path::Path;

pub struct Vendored {
    globs: Vec<Pattern>,
}

impl Vendored {
    pub fn new() -> Self {
        let globs = Vec::new();

        Self { globs }
    }

    pub fn add_dir<P: AsRef<Path>>(&mut self, dir: P) {
        let pattern = format!("{}/**", dir.as_ref().display());
        let pattern = Pattern::new(&pattern).unwrap();
        self.globs.push(pattern);
    }

    pub fn is_vendored<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.is_vendored_no_read(&filepath) || self.is_vendored_with_read(&filepath, contents)
    }

    fn is_vendored_no_read<P: AsRef<Path>>(&self, filepath: P) -> bool {
        filepath
            .as_ref()
            .components()
            .next()
            .map_or(false, |c| c.as_os_str() == "node_modules")
            || self
                .globs
                .iter()
                .any(|g| g.matches_path_with(filepath.as_ref(), GLOB_MATCH_OPTIONS))
    }

    fn is_vendored_with_read<P: AsRef<Path>>(&self, _filepath: P, _contents: &[u8]) -> bool {
        false
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
        case("node_modules", true)
    )]
    fn test_is_vendored_no_read(filepath: &str, expected: bool) {
        let vendored = Vendored::new();
        assert_eq!(vendored.is_vendored_no_read(filepath), expected);
    }
}
