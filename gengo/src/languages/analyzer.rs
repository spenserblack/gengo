//! Analyzes a language.
use super::{Category, Language, LANGUAGE_DEFINITIONS};
use indexmap::IndexMap;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::path::Path;

/// Analyzes and attempts to identify a language.
#[derive(Debug)]
pub struct Analyzers(Vec<Analyzer>);

impl Analyzers {
    fn iter(&self) -> impl Iterator<Item = &Analyzer> {
        self.0.iter()
    }

    /// First pass. Checks a file and returns the matching languages, and how
    /// they were matched.
    pub fn check(&self, filepath: &OsStr, contents: &[u8]) -> Vec<(&Language, Vec<&Matcher>)> {
        let matches = self.iter()
            .filter_map(|analyzer| {
                let matching_analyzers: Vec<_> = dbg!(analyzer.matchers.iter().filter(|matcher| matcher.matches(filepath, contents)).collect());
                if matching_analyzers.is_empty() {
                    None
                } else {
                    Some((&analyzer.language, matching_analyzers))
                }
            });
        matches.collect()
    }

    /// Creates analyzers from JSON.
    pub fn from_json(json: &str) -> Self {
        // TODO Return result instead of unwrapping.
        let languages: IndexMap<String, AnalyzerArgs> =
            serde_json::from_str(json).unwrap();
        Self::from_indexmap(languages)
    }

    /// Creates analyzers from YAML.
    pub fn from_yaml(yaml: &str) -> Self {
        // TODO Return result instead of unwrapping.
        let languages: IndexMap<String, AnalyzerArgs> =
            serde_yaml::from_str(yaml).unwrap();
        Self::from_indexmap(languages)
    }

    fn from_indexmap(languages: IndexMap<String, AnalyzerArgs>) -> Self {
        let analyzers = languages
            .into_iter()
            .map(|(name, args)| {
                let language = Language {
                    name,
                    category: args.category,
                    color: args.color,
                };
                let matchers = &args.matchers;
                let matchers = matchers.into();
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

impl Default for Analyzers {
    /// Create a new language analyzer with default values.
    fn default() -> Self {
        Self::from_json(LANGUAGE_DEFINITIONS)
    }
}

/// Used to match a programming language.
#[derive(Debug)]
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
#[derive(Debug)]
pub enum Matcher {
    Filepath(FilepathMatcher),
    Shebang(ShebangMatcher),
}

impl MatcherTrait for Matcher {
    fn matches(&self, filename: &OsStr, contents: &[u8]) -> bool {
        match self {
            Matcher::Filepath(matcher) => matcher.matches(filename, contents),
            Matcher::Shebang(matcher) => matcher.matches(filename, contents),
        }
    }
}

/// Matches a file path.
#[derive(Debug)]
pub struct FilepathMatcher {
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

/// Matches a shebang.
#[derive(Debug)]
pub struct ShebangMatcher {
    re: Regex,
}

impl ShebangMatcher {
    pub fn new<S: Display>(cmd: S) -> Self {
        let re = Regex::new(&format!(r"^#!(?:/usr(?:/local)?)?/bin/(?:env )?(?:{cmd})$")).unwrap();
        Self { re }
    }
}

impl MatcherTrait for ShebangMatcher {
    /// Checks if the file contents match a shebang by checking the first line of the contents.
    ///
    /// Does not read more than 100 bytes.
    fn matches(&self, _filename: &OsStr, contents: &[u8]) -> bool {
        let mut lines = contents.split(|&c| c == b'\n');
        let first_line = lines.next().unwrap_or_default();
        let first_line = if first_line.len() > 100 {
            &first_line[..100]
        } else {
            first_line
        };
        let first_line = String::from_utf8_lossy(first_line);
        self.re.is_match(&first_line)
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
    interpreter_pattern: Option<String>,
}

impl From<&AnalyzerArgMatchers> for Vec<Matcher> {
    fn from(matchers: &AnalyzerArgMatchers) -> Self {
        let filepath_matcher = if !matchers.extensions.is_empty()
            || !matchers.filenames.is_empty()
            || !matchers.patterns.is_empty()
        {
            Some(Matcher::Filepath(matchers.into()))
        } else {
            None
        };
        let shebang_matcher = matchers
            .interpreter_pattern
            .as_ref()
            .map(|p| Matcher::Shebang(ShebangMatcher::new(p)));
        [filepath_matcher, shebang_matcher]
            .into_iter()
            .filter_map(|m| m)
            .collect()
    }
}

impl From<&AnalyzerArgMatchers> for FilepathMatcher {
    fn from(matchers: &AnalyzerArgMatchers) -> Self {
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
        assert!(analyzer.matches(OsStr::new("foo.txt"), b""));
        assert!(!analyzer.matches(OsStr::new("foo.rs"), b""));
    }

    #[test]
    fn test_matches_filename() {
        let analyzer = FilepathMatcher::new(&[], &["LICENSE"], &[]);
        assert!(analyzer.matches(OsStr::new("LICENSE"), b""));
        assert!(!analyzer.matches(OsStr::new("Dockerfile"), b""));
    }

    #[test]
    fn test_matches_pattern() {
        let analyzer =
            FilepathMatcher::new::<&str>(&[], &[], &[r"^Makefile(?:\.[\w\d]+)?$".into()]);
        assert!(analyzer.matches(OsStr::new("Makefile"), b""));
        assert!(analyzer.matches(OsStr::new("Makefile.in"), b""));
        assert!(!analyzer.matches(OsStr::new("Cakefile"), b""));
    }

    #[test]
    fn test_matches_shebang() {
        let analyzer = ShebangMatcher::new(r"python3?");
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/bin/python\n"));
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/usr/bin/python\n"));
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/usr/local/bin/python\n"));
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/usr/bin/python3\n"));
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/usr/bin/env python\n"));
        assert!(!analyzer.matches(OsStr::new("foo.py"), b"#!/bin/sh\n"));
    }
}
