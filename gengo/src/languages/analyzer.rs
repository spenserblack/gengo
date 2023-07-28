//! Analyzes a language.
use super::{Category, Language, LANGUAGE_DEFINITIONS};
use indexmap::IndexMap;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// Analyzes and attempts to identify a language.
pub struct Analyzers(Vec<Analyzer>);

impl Analyzers {
    fn iter(&self) -> impl Iterator<Item = &Analyzer> {
        self.0.iter()
    }
}

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
                let matchers = args.matchers.into();
                let heuristics = args
                    .heuristics
                    .into_iter()
                    .map(|s| Regex::new(&s).unwrap())
                    .collect();
                Analyzer {
                    language,
                    matchers,
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
    matchers: Vec<Matcher>,
    heuristics: Vec<Regex>,
    priority: f32,
}

trait MatcherTrait {
    fn matches(&self, filename: &OsStr, contents: &[u8]) -> bool;
}

/// Checks if a file matches.
enum Matcher {
    Filepath(FilepathMatcher),
}

impl MatcherTrait for Matcher {
    fn matches(&self, filename: &OsStr, contents: &[u8]) -> bool {
        match self {
            Matcher::Filepath(matcher) => matcher.matches(filename, contents),
        }
    }
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
        let extensions = extensions.iter().map(Into::into).collect();
        let filenames = filenames.iter().map(Into::into).collect();
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

    pub fn matches_extension(&self, filename: &OsStr) -> bool {
        let extension = Path::new(filename).extension().unwrap_or_default();
        self.extensions.contains(extension)
    }

    pub fn matches_filename(&self, filename: &OsStr) -> bool {
        self.filenames
            .contains(Path::new(filename).file_name().unwrap_or_default())
    }

    pub fn matches_pattern(&self, filename: &OsStr) -> bool {
        let filename = if let Some(filename) = filename.to_str() {
            filename
        } else {
            return false;
        };
        self.patterns.iter().any(|p| p.is_match(filename))
    }
}

impl MatcherTrait for FilepathMatcher {
    fn matches(&self, filename: &OsStr, _contents: &[u8]) -> bool {
        self.matches_extension(&filename)
            || self.matches_filename(&filename)
            || self.matches_pattern(&filename)
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

impl From<AnalyzerArgMatchers> for Vec<Matcher> {
    fn from(matchers: AnalyzerArgMatchers) -> Self {
        let filepath_matcher = if !matchers.extensions.is_empty()
            || !matchers.filenames.is_empty()
            || !matchers.patterns.is_empty()
        {
            Some(Matcher::Filepath(matchers.into()))
        } else {
            None
        };
        [filepath_matcher]
            .into_iter()
            .filter_map(|m| m)
            .collect()
    }
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
        let analyzer = FilepathMatcher::new(&["txt"], &[], &[]);
        assert!(analyzer.matches_extension(OsStr::new("foo.txt")));
        assert!(!analyzer.matches_extension(OsStr::new("foo.rs")));
    }

    #[test]
    fn test_matches_filename() {
        let analyzer = FilepathMatcher::new(&[], &["LICENSE"], &[]);
        assert!(analyzer.matches_filename(OsStr::new("LICENSE")));
        assert!(!analyzer.matches_filename(OsStr::new("Dockerfile")));
    }

    #[test]
    fn test_matches_pattern() {
        let analyzer =
            FilepathMatcher::new::<&str>(&[], &[], &[r"^Makefile(?:\.[\w\d]+)?$".into()]);
        assert!(analyzer.matches_pattern(OsStr::new("Makefile")));
        assert!(analyzer.matches_pattern(OsStr::new("Makefile.in")));
        assert!(!analyzer.matches_pattern(OsStr::new("Cakefile")));
    }
}
