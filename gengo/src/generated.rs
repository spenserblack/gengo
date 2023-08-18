use glob::Pattern;
use std::path::Path;

pub struct Generated {
    globs: Vec<Pattern>,
}

impl Generated {
    pub fn new() -> Self {
        let globs = Self::globs();

        Self { globs }
    }

    pub fn is_generated<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.is_generated_no_read(&filepath) || self.is_generated_with_read(&filepath, contents)
    }

    fn is_generated_no_read<P: AsRef<Path>>(&self, filepath: P) -> bool {
        self.globs.iter().any(|g| g.matches_path(filepath.as_ref()))
    }

    fn is_generated_with_read<P: AsRef<Path>>(&self, _filepath: P, _contents: &[u8]) -> bool {
        false
    }

    fn globs() -> Vec<Pattern> {
        ["dist/**", "*.min.css", "*.min.js"]
            .into_iter()
            .map(|s| Pattern::new(s).unwrap())
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
        case("dist/something.js", true),
        case("src/something.rs", false),
        case("dist/subfolder/something.js", true),
        case("something.min.js", true),
        case("something.min.css", true),
        case("path/to/something.min.js", true),
        case("path/to/something.min.css", true)
    )]
    fn test_is_generated_no_read(filepath: &str, expected: bool) {
        let generated = Generated::new();
        assert_eq!(generated.is_generated_no_read(filepath), expected);
    }
}
