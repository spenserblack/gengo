//! Analyzes a language.
use super::{
    matcher::{Filename, FilepathPattern},
    Category, LanguageOld, LANGUAGE_DEFINITIONS,
};

use indexmap::IndexMap;

use regex::RegexSet;
use serde::Deserialize;
use std::error::Error;

use super::matcher::{Extension, Matcher, Shebang};
use std::path::Path;

/// Analyzes and attempts to identify a language.
#[derive(Debug)]
pub struct Analyzers(IndexMap<String, Analyzer>);

impl Analyzers {
    fn iter(&self) -> impl Iterator<Item = (&String, &Analyzer)> {
        self.0.iter()
    }

    /// Returns a language by name. This is case insensitive.
    pub fn get(&self, name: &str) -> Option<&LanguageOld> {
        let name = name.to_lowercase();
        self.0.get(&name).map(|a| &a.language)
    }

    /// Returns the analyzers that have matched by extension.
    pub fn by_extension(&self, filepath: impl AsRef<Path>) -> Found {
        let matches: Vec<_> = self
            .iter()
            .filter(|(_, a)| {
                a.matchers
                    .iter()
                    .filter_map(|m| {
                        if let Matcher::Extension(m) = m {
                            Some(m)
                        } else {
                            None
                        }
                    })
                    .any(|m| m.matches(&filepath))
            })
            .map(|(key, _)| key.to_owned())
            .collect();
        matches.into()
    }

    /// Returns the analyzers that have matched by filename.
    pub fn by_filename(&self, filepath: impl AsRef<Path>) -> Found {
        let matches: Vec<_> = self
            .iter()
            .filter(|(_, a)| {
                a.matchers
                    .iter()
                    .filter_map(|m| {
                        if let Matcher::Filename(m) = m {
                            Some(m)
                        } else {
                            None
                        }
                    })
                    .any(|m| m.matches(&filepath))
            })
            .map(|(key, _)| key.to_owned())
            .collect();
        matches.into()
    }

    /// Returns the analyzers that have matched by filepath pattern.
    pub fn by_filepath_pattern(&self, filepath: impl AsRef<Path>) -> Found {
        let matches: Vec<_> = self
            .iter()
            .filter(|(_, a)| {
                a.matchers
                    .iter()
                    .filter_map(|m| {
                        if let Matcher::FilepathPattern(m) = m {
                            Some(m)
                        } else {
                            None
                        }
                    })
                    .any(|m| m.matches(&filepath))
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
    pub fn simple(&self, filepath: impl AsRef<Path>, contents: &[u8]) -> Found {
        let matches = self.by_shebang(contents);
        if !matches.is_empty() {
            return matches;
        }
        let matches = self.by_filename(&filepath);
        if !matches.is_empty() {
            return matches;
        }
        let matches = self.by_filepath_pattern(&filepath);
        if !matches.is_empty() {
            return matches;
        }
        self.by_extension(&filepath)
    }

    /// Second pass over a file to determine the language.
    ///
    /// If a single language isn't found, narrows down the matches by heuristics.
    /// If none of the found heuristics match, returns the original matches.
    ///
    /// Use `limit` to limit the number of bytes to read to match to heuristics.
    pub fn with_heuristics(
        &self,
        filepath: impl AsRef<Path>,
        contents: &[u8],
        limit: usize,
    ) -> Found {
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
            .filter(|(_, a)| a.heuristics.is_match(contents))
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
    /// # use std::path::Path;
    /// use gengo::Analyzers;
    ///
    /// // Minified JSON of the above definition.
    /// const DEFINITIONS: &str = r##"{"Rust":{"category":"programming","color":"#FF4400","matchers":{"extensions":["rs"]}}}"##;
    /// let analyzers = Analyzers::from_json(DEFINITIONS).unwrap();
    /// let filename = Path::new("main.rs");
    /// let contents = b"fn main() {}";
    /// let limit = 1 << 20; // 1 MB
    /// let language = analyzers.pick(filename, contents, limit).unwrap();
    /// assert_eq!(language.name(), "Rust");
    /// ```
    pub fn pick(
        &self,
        filepath: impl AsRef<Path>,
        contents: &[u8],
        limit: usize,
    ) -> Option<&LanguageOld> {
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
        matches.first().map(|a| &a.language)
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
                let language = LanguageOld {
                    name,
                    category: args.category,
                    color: args.color,
                };
                let matchers = &args.matchers;
                let matchers = matchers.into();
                let heuristics = RegexSet::new(args.heuristics)?;
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
    language: LanguageOld,
    matchers: Vec<Matcher>,
    heuristics: RegexSet,
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
        let extension_matcher = if !matchers.extensions.is_empty() {
            Some(Matcher::Extension(matchers.into()))
        } else {
            None
        };
        let filename_matcher = if !matchers.filenames.is_empty() {
            Some(Matcher::Filename(Filename::new(&matchers.filenames)))
        } else {
            None
        };
        let filepath_pattern_matcher = if !matchers.patterns.is_empty() {
            Some(Matcher::FilepathPattern(FilepathPattern::new(
                &matchers.patterns,
            )))
        } else {
            None
        };
        let shebang_matcher = if matchers.interpreters.is_empty() {
            None
        } else {
            let shebang_matcher = Shebang::new(&matchers.interpreters);
            Some(Matcher::Shebang(shebang_matcher))
        };
        [
            extension_matcher,
            filename_matcher,
            filepath_pattern_matcher,
            shebang_matcher,
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl From<&AnalyzerArgMatchers> for Extension {
    fn from(matchers: &AnalyzerArgMatchers) -> Self {
        Self::new(&matchers.extensions)
    }
}

impl From<&AnalyzerArgMatchers> for Filename {
    fn from(matchers: &AnalyzerArgMatchers) -> Self {
        Self::new(&matchers.filenames)
    }
}

impl From<&AnalyzerArgMatchers> for FilepathPattern {
    fn from(matchers: &AnalyzerArgMatchers) -> Self {
        Self::new(&matchers.patterns)
    }
}
