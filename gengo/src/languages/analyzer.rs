//! Analyzes a language.
use super::{Category, Language, LANGUAGE_DEFINITIONS};
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use serde::Deserialize;
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

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Analyzer> {
        self.0.iter_mut()
    }

    /// First pass. Checks a file and returns the matching languages, and how
    /// they were matched.
    pub fn check(&mut self, filepath: &OsStr, contents: &[u8]) -> Vec<(&Language, Vec<&Matcher>)> {
        let matches = self.iter_mut().filter_map(|analyzer| {
            let mut matching_analyzers = Vec::with_capacity(analyzer.matchers.len());
            for matcher in &mut analyzer.matchers {
                if matcher.matches(filepath, contents) {
                    matching_analyzers.push(&*matcher);
                }
            }
            // NOTE Shadow with immutable variable
            let matching_analyzers = matching_analyzers;
            if matching_analyzers.is_empty() {
                None
            } else {
                Some((&analyzer.language, matching_analyzers))
            }
        });
        let matches: Vec<_> = matches.collect();
        matches
    }

    /// Creates analyzers from JSON.
    pub fn from_json(json: &str) -> Self {
        // TODO Return result instead of unwrapping.
        let languages: IndexMap<String, AnalyzerArgs> = serde_json::from_str(json).unwrap();
        Self::from_indexmap(languages)
    }

    /// Creates analyzers from YAML.
    pub fn from_yaml(yaml: &str) -> Self {
        // TODO Return result instead of unwrapping.
        let languages: IndexMap<String, AnalyzerArgs> = serde_yaml::from_str(yaml).unwrap();
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
    /// A value between `0` and `100` that determines the priority of a match.
    priority: u8,
}

trait MatcherTrait {
    /// Checks if a file matches.
    ///
    /// `self` is mut because some matchers may need to be compiled lazily.
    fn matches(&mut self, filename: &OsStr, contents: &[u8]) -> bool;
}

/// Checks if a file matches.
#[derive(Debug)]
#[non_exhaustive]
pub enum Matcher {
    Filepath(FilepathMatcher),
    Shebang(ShebangMatcher),
}

impl MatcherTrait for Matcher {
    fn matches(&mut self, filename: &OsStr, contents: &[u8]) -> bool {
        match self {
            Matcher::Filepath(matcher) => matcher.matches(filename, contents),
            Matcher::Shebang(matcher) => matcher.matches(filename, contents),
        }
    }
}

/// Matches a file path.
#[derive(Debug)]
pub struct FilepathMatcher {
    extensions: IndexSet<OsString>,
    filenames: IndexSet<OsString>,
    patterns: Vec<Regex>,
}

impl FilepathMatcher {
    /// Create a new filepath matcher.
    fn new<S: AsRef<OsStr>>(extensions: &[S], filenames: &[S], patterns: &[String]) -> Self {
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
    fn matches(&mut self, filename: &OsStr, _contents: &[u8]) -> bool {
        self.matches_extension(&filename)
            || self.matches_filename(&filename)
            || self.matches_pattern(&filename)
    }
}

/// Matches a shebang.
#[derive(Debug)]
pub struct ShebangMatcher {
    matchers: Vec<LazyShebangMatcher>,
}

#[derive(Debug)]
enum LazyShebangMatcher {
    Compiled(Regex),
    Uncompiled(String),
}

impl ShebangMatcher {
    fn new<S: Display>(cmd: &[S]) -> Self {
        let matchers = cmd
            .iter()
            .map(|s| LazyShebangMatcher::Uncompiled(s.to_string()))
            .collect();
        Self { matchers }
    }
}

impl MatcherTrait for ShebangMatcher {
    /// Checks if the file contents match a shebang by checking the first line of the contents.
    ///
    /// Does not read more than 100 bytes.
    fn matches(&mut self, _filename: &OsStr, contents: &[u8]) -> bool {
        let mut lines = contents.split(|&c| c == b'\n');
        let first_line = lines.next().unwrap_or_default();
        let first_line = if first_line.len() > 100 {
            &first_line[..100]
        } else {
            first_line
        };
        let first_line = String::from_utf8_lossy(first_line);
        self.matchers
            .iter_mut()
            .map(|m| m.compile())
            .any(|matcher| {
                if let LazyShebangMatcher::Compiled(re) = matcher {
                    re.is_match(&first_line)
                } else {
                    unreachable!("matcher should be compiled")
                }
            })
    }
}

impl LazyShebangMatcher {
    fn compile(&mut self) -> &Self {
        match self {
            LazyShebangMatcher::Compiled(_) => self,
            LazyShebangMatcher::Uncompiled(cmd) => {
                let re = Regex::new(&format!(r"^#!(?:/usr(?:/local)?)?/bin/(?:env )?(?:{cmd})$"))
                    .unwrap();
                *self = LazyShebangMatcher::Compiled(re);
                self
            }
        }
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
    priority: u8,
}

fn default_priority() -> u8 {
    50
}

#[derive(Debug, Deserialize)]
struct AnalyzerArgMatchers {
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    filenames: Vec<String>,
    #[serde(default)]
    patterns: Vec<String>,
    #[serde(default)]
    interpreters: Vec<String>,
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
        let shebang_matcher = if matchers.interpreters.is_empty() {
            None
        } else {
            let shebang_matcher = ShebangMatcher::new(&matchers.interpreters);
            Some(Matcher::Shebang(shebang_matcher))
        };
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
        let mut analyzer = FilepathMatcher::new(&["txt"], &[], &[]);
        assert!(analyzer.matches(OsStr::new("foo.txt"), b""));
        assert!(!analyzer.matches(OsStr::new("foo.rs"), b""));
    }

    #[test]
    fn test_matches_filename() {
        let mut analyzer = FilepathMatcher::new(&[], &["LICENSE"], &[]);
        assert!(analyzer.matches(OsStr::new("LICENSE"), b""));
        assert!(!analyzer.matches(OsStr::new("Dockerfile"), b""));
    }

    #[test]
    fn test_matches_pattern() {
        let mut analyzer =
            FilepathMatcher::new::<&str>(&[], &[], &[r"^Makefile(?:\.[\w\d]+)?$".into()]);
        assert!(analyzer.matches(OsStr::new("Makefile"), b""));
        assert!(analyzer.matches(OsStr::new("Makefile.in"), b""));
        assert!(!analyzer.matches(OsStr::new("Cakefile"), b""));
    }

    #[test]
    fn test_matches_shebang() {
        let mut analyzer = ShebangMatcher::new(&["python", "python3"]);
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/bin/python\n"));
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/usr/bin/python\n"));
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/usr/local/bin/python\n"));
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/usr/bin/python3\n"));
        assert!(analyzer.matches(OsStr::new("foo.py"), b"#!/usr/bin/env python\n"));
        assert!(!analyzer.matches(OsStr::new("foo.py"), b"#!/bin/sh\n"));
    }

    #[test]
    fn test_shebang_lazy_compile() {
        let mut analyzer = ShebangMatcher::new(&["sh", "bash"]);
        analyzer.matches(OsStr::new("foo.sh"), b"#!/bin/sh\n");
        assert!(matches!(
            analyzer.matchers[0],
            LazyShebangMatcher::Compiled(_)
        ));
        assert!(matches!(
            analyzer.matchers[1],
            LazyShebangMatcher::Uncompiled(_)
        ));
    }
}
