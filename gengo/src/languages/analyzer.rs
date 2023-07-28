//! Analyzes a language.
use super::{Category, Language, LANGUAGE_DEFINITIONS};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use indexmap::IndexMap;
use std::path::Path;

pub struct Analyzers(Vec<Analyzer>);

impl Default for Analyzers {
    /// Create a new language analyzer with default values.
    fn default() -> Self {
        let languages: IndexMap<String, AnalyzerArgs> =
            serde_json::from_str(LANGUAGE_DEFINITIONS).unwrap();
        let analyzers = languages
            .into_iter()
            .map(|(name, args)| {
                let language = Language {
                    name,
                    category: args.category,
                    color: args.color,
                };
                let matcher = args.matchers.into();
                let heuristics = args
                    .heuristics
                    .into_iter()
                    .map(|s| Regex::new(&s).unwrap())
                    .collect();
                Analyzer {
                    language,
                    matcher,
                    heuristics,
                    priority: args.priority,
                }
            })
            .collect();
        Self(analyzers)
    }
}

/// Used to match a programming language.
struct Analyzer {
    language: Language,
    matcher: FilepathMatcher,
    heuristics: Vec<Regex>,
    priority: f32,
}

/// Matches a file path.
struct FilepathMatcher {
    extensions: HashSet<OsString>,
    filenames: HashSet<OsString>,
    patterns: Vec<Regex>,
}

impl FilepathMatcher {
    /// Create a new filepath matcher.
    pub fn new<S: AsRef<OsStr>>(extensions: &[S], filenames: &[S], patterns: &[String]) -> Self {
        let extensions = extensions
            .iter()
            .map(Into::into)
            .collect();
        let filenames = filenames
            .iter()
            .map(Into::into)
            .collect();
        let patterns = patterns
            .iter()
            .map(|s| Regex::new(s.as_ref()).unwrap())
            .collect();
        Self {
            extensions,
            filenames,
            patterns,
        }

    }

    pub fn matches_extension(&self, filename: &str) -> bool {
        let extension = Path::new(filename).extension().unwrap_or_default();
        self.extensions.contains(extension)
    }

    pub fn matches_filename(&self, filename: &str) -> bool {
        self.filenames
            .contains(Path::new(filename).file_name().unwrap_or_default())
    }
}

#[derive(Debug, Deserialize)]
struct AnalyzerArgs {
    category: Category,
    color: String,
    matchers: AnalyzerArgMatchers,
    #[serde(default)]
    heuristics: Vec<String>,
    #[serde(default = "default_priority")]
    priority: f32,
}

fn default_priority() -> f32 {
    0.5
}

#[derive(Debug, Deserialize)]
struct AnalyzerArgMatchers {
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    filenames: Vec<String>,
    #[serde(default)]
    patterns: Vec<String>,
}

impl From<AnalyzerArgMatchers> for FilepathMatcher {
    fn from(matchers: AnalyzerArgMatchers) -> Self {
        Self::new(
            &matchers.extensions,
            &matchers.filenames,
            &matchers.patterns,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_extension() {
        let analyzer = FilepathMatcher::new(
            &["txt"],
            &[],
            &[],
        );
        assert!(analyzer.matches_extension("foo.txt"));
        assert!(!analyzer.matches_extension("foo.rs"));
    }

    #[test]
    fn test_matches_filename() {
        let analyzer = FilepathMatcher::new(
            &[],
            &["LICENSE"],
            &[],
        );
        assert!(analyzer.matches_filename("LICENSE"));
        assert!(!analyzer.matches_filename("Dockerfile"));
    }
}
