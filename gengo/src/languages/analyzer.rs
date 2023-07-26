/// Analyzes a language.
use super::{Category, Language};
use std::collections::HashSet;
use regex::Regex;
use std::path::Path;
use std::ffi::{OsStr, OsString};

pub struct Analyzers(Vec<Analyzer>);

impl Default for Analyzers {
    /// Create a new language analyzer with default values.
    fn default() -> Self {
        Self(include!(concat!(env!("OUT_DIR"), "/analyzer.rs")))
    }
}
pub struct Analyzer {
    language: Language,
    category: Category,
    color: String,
    extensions: HashSet<OsString>,
    filenames: HashSet<OsString>,
    patterns: Vec<Regex>,
    heuristics: Vec<Regex>,
    priority: f32,
}

impl Analyzer {
    /// Create a new language analyzer.
    fn new(
        language: Language,
        category: Category,
        color: &str,
        extensions: &[&str],
        filenames: &[&str],
        patterns: &[&str],
        heuristics: &[&str],
        priority: f32,
    ) -> Self {
        let extensions = extensions.iter().map(|s| s.into()).collect();
        let filenames = filenames.iter().map(|s| s.into()).collect();
        // TODO Handle regex compile failures.
        let patterns = patterns.iter().map(|s| Regex::new(s).unwrap()).collect();
        let heuristics = heuristics.iter().map(|s| Regex::new(s).unwrap()).collect();
        Self {
            language,
            category,
            color: color.to_string(),
            extensions,
            filenames,
            patterns,
            heuristics,
            priority,
        }
    }

    pub fn matches_extension(&self, filename: &str) -> bool {
        let extension = Path::new(filename).extension().unwrap_or_default();
        self.extensions.contains(extension)
    }

    pub fn matches_filename(&self, filename: &str) -> bool {
        self.filenames.contains(Path::new(filename).file_name().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_extension() {
        let analyzer = Analyzer::new(
            Language::PlainText,
            Category::Prose,
            "#000000",
            &["txt"],
            &[],
            &[],
            &[],
            0.5,
        );
        assert!(analyzer.matches_extension("foo.txt"));
        assert!(!analyzer.matches_extension("foo.rs"));
    }

    #[test]
    fn test_matches_filename() {
        let analyzer = Analyzer::new(
            Language::PlainText,
            Category::Prose,
            "#000000",
            &[],
            &["LICENSE"],
            &[],
            &[],
            0.5,
        );
        assert!(analyzer.matches_filename("LICENSE"));
        assert!(!analyzer.matches_filename("Dockerfile"));
    }
}
