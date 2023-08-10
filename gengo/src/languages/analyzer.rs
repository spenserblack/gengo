//! Analyzes a language.
use super::{Category, Language, LANGUAGE_DEFINITIONS};
use indexmap::{IndexMap, IndexSet};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::path::Path;

/// Analyzes and attempts to identify a language.
#[derive(Debug)]
pub struct Analyzers(IndexMap<String, Analyzer>);

impl Analyzers {
    fn iter(&self) -> impl Iterator<Item = (&String, &Analyzer)> {
        self.0.iter()
    }

    /// Returns a language by name. This is case insensitive.
    pub fn get(&self, name: &str) -> Option<&Language> {
        let name = name.to_lowercase();
        self.0.get(&name).map(|a| &a.language)
    }

    /// Returns the analyzers that have matched by filepath.
    pub fn by_filepath(&self, filepath: &OsStr) -> Found {
        let matches: Vec<_> = self
            .iter()
            .filter(|(_, a)| {
                a.matchers
                    .iter()
                    .filter_map(|m| {
                        if let Matcher::Filepath(m) = m {
                            Some(m)
                        } else {
                            None
                        }
                    })
                    .any(|m| m.matches(filepath))
            })
            .map(|(key, _)| key.to_owned())
            .collect();
        matches.into()
    }

    /// Returns the analyzers that have matched by shebang (`#!`).
    pub fn by_shebang(&self, contents: &[u8]) -> Found {
        let matches: Vec<_> = self
            .iter()
            .filter(|(_, a)| {
                a.matchers
                    .iter()
                    .filter_map(|m| {
                        if let Matcher::Shebang(m) = m {
                            Some(m)
                        } else {
                            None
                        }
                    })
                    .any(|m| m.matches(contents))
            })
            .map(|(key, _)| key.to_owned())
            .collect();
        matches.into()
    }

    /// First pass over a file to determine the language.
    ///
    /// It attempts to identify the file in this order:
    /// 1. by shebang (`#!`)
    /// 2. by filepath
    pub fn simple(&self, filepath: &OsStr, contents: &[u8]) -> Found {
        let matches = self.by_shebang(contents);
        if !matches.is_empty() {
            return matches;
        }
        self.by_filepath(filepath)
    }

    /// Second pass over a file to determine the language.
    ///
    /// If a single language isn't found, narrows down the matches by heuristics.
    /// If none of the found heuristics match, returns the original matches.
    ///
    /// Use `limit` to limit the number of bytes to read to match to heuristics.
    pub fn with_heuristics(&self, filepath: &OsStr, contents: &[u8], limit: usize) -> Found {
        let contents = if contents.len() > limit {
            &contents[..limit]
        } else {
            contents
        };
        let matches = self.simple(filepath, contents);
        let matches = match matches {
            Found::None | Found::One(_) => return matches,
            Found::Multiple(names) => names,
        };
        let contents: &str = std::str::from_utf8(contents).unwrap_or_default();
        let heuristic_matches: Vec<_> = matches
            .iter()
            .map(|key| {
                let a = self.0.get(key).unwrap();
                (key, a)
            })
            .filter(|(_, a)| a.heuristics.iter().any(|h| h.is_match(contents)))
            .map(|(key, _)| key)
            .collect();
        if heuristic_matches.is_empty() {
            return matches.into();
        }
        heuristic_matches
            .into_iter()
            .cloned()
            .collect::<Vec<String>>()
            .into()
    }

    /// Picks the best language to match to a file.
    ///
    /// Matches are first attempted by shebang. If there are no matches, then
    /// matches are attempted by filepath.
    ///
    /// After this, if there are multiple languages, then matches are narrowed
    /// down using heuristics. `limit` is used here to limit the number of
    /// bytes to read to match to heuristics.
    ///
    /// Finally, after this, if there are *still* multiple matching languages,
    /// then a language is chosen from community-driven priority.
    ///
    /// # Example
    ///
    /// Given the following simple definition, we can identify Rust code.
    ///
    /// ```yaml
    /// Rust:
    ///   category: programming
    ///   color: "#FF4400"
    ///   matchers:
    ///     extensions:
    ///       - rs
    /// ```
    ///
    /// ```
    /// # use std::ffi::OsStr;
    /// use gengo::Analyzers;
    ///
    /// // Minified JSON of the above definition.
    /// const DEFINITIONS: &str = r##"{"Rust":{"category":"programming","color":"#FF4400","matchers":{"extensions":["rs"]}}}"##;
    /// let analyzers = Analyzers::from_json(DEFINITIONS).unwrap();
    /// let filename = OsStr::new("main.rs");
    /// let contents = b"fn main() {}";
    /// let limit = 1 << 20; // 1 MB
    /// let language = analyzers.pick(filename, contents, limit).unwrap();
    /// assert_eq!(language.name(), "Rust");
    /// ```
    pub fn pick(&self, filepath: &OsStr, contents: &[u8], limit: usize) -> Option<&Language> {
        let matches = self.with_heuristics(filepath, contents, limit);
        let matches = match matches {
            Found::None => return None,
            Found::One(name) => return self.0.get(&name).map(|a| &a.language),
            Found::Multiple(names) => names,
        };
        let matches = {
            let mut matches: Vec<_> = matches
                .into_iter()
                .map(|name| self.0.get(&name).unwrap())
                .collect();
            matches.sort_by_key(|a| a.priority);
            matches.reverse();
            matches
        };
        matches.get(0).map(|a| &a.language)
    }

    /// Creates analyzers from JSON.
    pub fn from_json(json: &str) -> Result<Self, Box<dyn Error>> {
        let languages: IndexMap<String, AnalyzerArgs> = serde_json::from_str(json)?;
        Self::from_indexmap(languages)
    }

    /// Creates analyzers from YAML.
    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn Error>> {
        let languages: IndexMap<String, AnalyzerArgs> = serde_yaml::from_str(yaml)?;
        Self::from_indexmap(languages)
    }

    fn from_indexmap(languages: IndexMap<String, AnalyzerArgs>) -> Result<Self, Box<dyn Error>> {
        let analyzers = languages
            .into_iter()
            .map(|(name, args)| {
                let key = name.to_lowercase();
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
                    .map(|s| Ok(Regex::new(&s)?))
                    .collect::<Result<_, Box<dyn Error>>>()?;
                let analyzer = Analyzer {
                    language,
                    matchers,
                    heuristics,
                    priority: args.priority,
                };
                Ok((key, analyzer))
            })
            .collect::<Result<_, Box<dyn Error>>>()?;
        Ok(Self(analyzers))
    }
}

impl Default for Analyzers {
    /// Create a new language analyzer with default values.
    fn default() -> Self {
        Self::from_json(LANGUAGE_DEFINITIONS).unwrap()
    }
}

/// Used to match a programming language.
#[derive(Clone, Debug)]
pub struct Analyzer {
    language: Language,
    matchers: Vec<Matcher>,
    heuristics: Vec<Regex>,
    /// A value between `0` and `100` that determines the priority of a match.
    priority: u8,
}

/// The result of an analysis. Either multiple results, one result, or no result.
#[derive(Debug)]
pub enum Found {
    None,
    /// A key to get a language.
    One(String),
    /// Multiple keys to get languages. The contained vec should always have length
    /// of at least 2.
    Multiple(Vec<String>),
}

impl Found {
    /// Returns the first language.
    pub fn first(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::One(name) => Some(name),
            Self::Multiple(names) => names.first().map(|s| s.as_str()),
        }
    }

    /// Gets the length of the analysis.
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::One(_) => 1,
            Self::Multiple(names) => names.len(),
        }
    }

    /// Checks if the results are empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl From<Vec<String>> for Found {
    fn from(names: Vec<String>) -> Self {
        match names.len() {
            0 => Self::None,
            1 => Self::One(names.into_iter().next().unwrap()),
            _ => Self::Multiple(names),
        }
    }
}

impl IntoIterator for Found {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::None => Vec::with_capacity(0).into_iter(),
            Self::One(name) => vec![name].into_iter(),
            Self::Multiple(names) => names.into_iter(),
        }
    }
}

trait MatcherTrait {
    /// Checks if a file matches.
    ///
    /// `self` is mut because some matchers may need to be compiled lazily.
    fn matches(&self, filename: &OsStr, contents: &[u8]) -> bool;
}

/// Checks if a file matches.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Matcher {
    Filepath(FilepathMatcher),
    Shebang(ShebangMatcher),
}

impl MatcherTrait for Matcher {
    fn matches(&self, filename: &OsStr, contents: &[u8]) -> bool {
        match self {
            Matcher::Filepath(matcher) => FilepathMatcher::matches(matcher, filename),
            Matcher::Shebang(matcher) => matcher.matches(contents),
        }
    }
}

/// Matches a file path.
#[derive(Clone, Debug)]
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

    pub fn matches(&self, filename: &OsStr) -> bool {
        self.matches_extension(filename)
            || self.matches_filename(filename)
            || self.matches_pattern(filename)
    }
}

impl MatcherTrait for FilepathMatcher {
    fn matches(&self, filename: &OsStr, _contents: &[u8]) -> bool {
        FilepathMatcher::matches(self, filename)
    }
}

/// Matches a shebang.
#[derive(Clone, Debug)]
pub struct ShebangMatcher {
    interpreters: IndexSet<String>,
}

impl ShebangMatcher {
    const MAX_SHEBANG_LENGTH: usize = 50;

    fn new<S: Display>(interpreters: &[S]) -> Self {
        let interpreters = interpreters.iter().map(|s| s.to_string()).collect();
        Self { interpreters }
    }

    /// Checks if the file contents match a shebang by checking the first line of the contents.
    ///
    /// Does not read more than 100 bytes.
    pub fn matches(&self, contents: &[u8]) -> bool {
        let mut lines = contents.split(|&c| c == b'\n');
        let first_line = lines.next().unwrap_or_default();
        // Check that the first line is a shebang
        if first_line.len() < 2 || first_line[0] != b'#' || first_line[1] != b'!' {
            return false;
        }
        let first_line = if first_line.len() > Self::MAX_SHEBANG_LENGTH {
            &first_line[..Self::MAX_SHEBANG_LENGTH]
        } else {
            first_line
        };
        let first_line = String::from_utf8_lossy(first_line);
        // NOTE Handle trailing spaces, `\r`, etc.
        let first_line = first_line.trim_end();
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^#!(?:/usr(?:/local)?)?/bin/(?:env )?([\w\d]+)\r?$").unwrap()
        });

        RE.captures(first_line)
            .and_then(|c| c.get(1))
            .map_or(false, |m| {
                let interpreter = m.as_str();
                self.interpreters.contains(interpreter)
            })
    }
}

impl MatcherTrait for ShebangMatcher {
    /// Checks if the file contents match a shebang by checking the first line of the contents.
    ///
    /// Does not read more than 100 bytes.
    fn matches(&self, _filename: &OsStr, contents: &[u8]) -> bool {
        ShebangMatcher::matches(self, contents)
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
            .flatten()
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
        assert!(analyzer.matches(OsStr::new("foo.txt")));
        assert!(!analyzer.matches(OsStr::new("foo.rs")));
    }

    #[test]
    fn test_matches_filename() {
        let analyzer = FilepathMatcher::new(&[], &["LICENSE"], &[]);
        assert!(analyzer.matches(OsStr::new("LICENSE")));
        assert!(!analyzer.matches(OsStr::new("Dockerfile")));
    }

    #[test]
    fn test_matches_pattern() {
        let analyzer =
            FilepathMatcher::new::<&str>(&[], &[], &[r"^Makefile(?:\.[\w\d]+)?$".into()]);
        assert!(analyzer.matches(OsStr::new("Makefile")));
        assert!(analyzer.matches(OsStr::new("Makefile.in")));
        assert!(!analyzer.matches(OsStr::new("Cakefile")));
    }

    #[test]
    fn test_matches_shebang() {
        let analyzer = ShebangMatcher::new(&["python", "python3"]);
        assert!(analyzer.matches(b"#!/bin/python\n"));
        assert!(analyzer.matches(b"#!/usr/bin/python\n"));
        assert!(analyzer.matches(b"#!/usr/local/bin/python\n"));
        assert!(analyzer.matches(b"#!/usr/bin/python3\n"));
        assert!(analyzer.matches(b"#!/usr/bin/env python\n"));
        assert!(!analyzer.matches(b"#!/bin/sh\n"));
    }
}
