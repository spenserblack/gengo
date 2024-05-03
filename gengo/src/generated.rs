use super::GLOB_MATCH_OPTIONS;
use glob::Pattern;
use std::collections::HashSet;
use std::path::Path;

pub struct Generated {
    filenames: HashSet<&'static str>,
    globs: Vec<Pattern>,
}

impl Generated {
    pub fn new() -> Self {
        let filenames = Self::filenames();
        let globs = Self::globs();

        Self { filenames, globs }
    }

    pub fn is_generated(&self, filepath: impl AsRef<Path>, contents: &[u8]) -> bool {
        self.is_generated_no_read(&filepath) || self.is_generated_with_read(&filepath, contents)
    }

    fn is_generated_no_read(&self, filepath: impl AsRef<Path>) -> bool {
        self.matches_filenames(&filepath) || self.matches_globs(&filepath)
    }

    fn is_generated_with_read(&self, _filepath: impl AsRef<Path>, contents: &[u8]) -> bool {
        self.likely_minified(contents)
    }

    fn matches_filenames(&self, filepath: impl AsRef<Path>) -> bool {
        let filename = filepath.as_ref().file_name().and_then(|f| f.to_str());
        filename.map_or(false, |f| self.filenames.contains(f))
    }

    fn matches_globs(&self, filepath: impl AsRef<Path>) -> bool {
        self.globs
            .iter()
            .any(|g| g.matches_path_with(filepath.as_ref(), GLOB_MATCH_OPTIONS))
    }

    fn likely_minified(&self, contents: &[u8]) -> bool {
        // NOTE If the first 10 lines are really long, it's probably minified.
        contents
            .split(|&b| b == b'\n')
            .take(10)
            .any(|line| line.len() > 250)
    }

    fn filenames() -> HashSet<&'static str> {
        HashSet::from_iter(["gradlew", "gradlew.bat"])
    }

    fn globs() -> Vec<Pattern> {
        [
            "dist/**",
            "**/*.min.css",
            "**/*.min.js",
            ".yarn/**",
            "**/migrations/*.py",
        ]
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
        case("path/to/something.min.css", true),
        case(".yarn/releases/yarn-1.2.3.cjs", true),
        case("migrations/0001_initial.py", true),
        case("myapp/migrations/0001_initial.py", true),
        case("gradlew", true),
        case("gradlew.bat", true)
    )]
    fn test_is_generated_no_read(filepath: &str, expected: bool) {
        let generated = Generated::new();
        assert_eq!(generated.is_generated_no_read(filepath), expected);
    }

    #[test]
    fn test_likely_minified() {
        let generated = Generated::new();
        let header: Vec<u8> = b"/*!\n  * This is my license etc etc\n */".to_vec();
        let contents = b"console.log('hello, world!');".repeat(50);
        let contents = [header, contents].concat();
        assert!(generated.likely_minified(&contents));
    }
}
