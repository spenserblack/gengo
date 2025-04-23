use crate::GLOB_MATCH_OPTIONS;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;

macro_rules! _include {
    ($path:literal) => {
        include!(concat!(env!("OUT_DIR"), "/languages/", $path));
    };
}

_include!("language.rs");
_include!("category_mixin.rs");
_include!("name_mixin.rs");
_include!("parse_variant_mixin.rs");
_include!("color_hex_mixin.rs");
_include!("color_rgb_mixin.rs");
_include!("nerd_font_glyph_mixin.rs");
_include!("priority_mixin.rs");
_include!("from_extension_mixin.rs");
_include!("from_filename_mixin.rs");
_include!("from_interpreter_mixin.rs");
_include!("glob_mappings_mixin.rs");
_include!("heuristic_mappings_mixin.rs");

impl Language {
    /// Gets languages from a path's extension.
    fn from_path_extension(path: impl AsRef<Path>) -> Vec<Self> {
        let extension = path.as_ref().extension().and_then(|ext| ext.to_str());
        extension.map_or(vec![], Self::from_extension)
    }

    /// Gets languages from a path's filename.
    fn from_path_filename(path: impl AsRef<Path>) -> Vec<Self> {
        let filename = path
            .as_ref()
            .file_name()
            .and_then(|filename| filename.to_str());
        filename.map_or(vec![], Self::from_filename)
    }

    /// Gets languages by a shebang.
    fn from_shebang(contents: &[u8]) -> Vec<Self> {
        const MAX_SHEBANG_LENGTH: usize = 50;

        let mut lines = contents.split(|&c| c == b'\n');
        let first_line = lines.next().unwrap_or_default();
        if first_line.len() < 2 || first_line[0] != b'#' || first_line[1] != b'!' {
            return vec![];
        }
        let first_line = if first_line.len() > MAX_SHEBANG_LENGTH {
            &first_line[..MAX_SHEBANG_LENGTH]
        } else {
            first_line
        };
        let first_line = String::from_utf8_lossy(first_line);
        // NOTE Handle trailing spaces, `\r`, etc.
        let first_line = first_line.trim_end();

        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^#!(?:/usr(?:/local)?)?/bin/(?:env\s+)?([\w\d]+)\r?$").unwrap()
        });

        RE.captures(first_line)
            .and_then(|c| c.get(1))
            .map_or(vec![], |m| {
                let interpreter = m.as_str();
                Self::from_interpreter(interpreter)
            })
    }

    /// Gets the languages that match a glob pattern.
    pub fn from_glob(path: impl AsRef<Path>) -> Vec<Self> {
        let path = path.as_ref();

        struct GlobMapping {
            patterns: Vec<glob::Pattern>,
            language: Language,
        }
        static GLOB_MAPPINGS: LazyLock<Vec<GlobMapping>> = LazyLock::new(|| {
            Language::glob_mappings()
                .into_iter()
                .map(|(patterns, language)| {
                    let patterns = patterns
                        .into_iter()
                        .map(|pattern| glob::Pattern::new(pattern).unwrap())
                        .collect();
                    GlobMapping { patterns, language }
                })
                .collect()
        });

        GLOB_MAPPINGS
            .iter()
            .filter(|gm| {
                gm.patterns
                    .iter()
                    .any(|p| p.matches_path_with(path.as_ref(), GLOB_MATCH_OPTIONS))
            })
            .map(|gm| gm.language)
            .collect()
    }

    /// Filters an iterable of languages by heuristics.
    fn filter_by_heuristics(languages: &[Self], contents: &str) -> Vec<Self> {
        static HEURISTICS: LazyLock<HashMap<Language, Vec<Regex>>> = LazyLock::new(|| {
            Language::heuristic_mappings()
                .into_iter()
                .map(|(language, patterns)| {
                    let patterns = patterns
                        .into_iter()
                        .map(|pattern| Regex::new(pattern).unwrap())
                        .collect();
                    (language, patterns)
                })
                .collect()
        });

        languages
            .iter()
            .filter(|language| {
                HEURISTICS
                    .get(language)
                    .is_some_and(|heuristics| heuristics.iter().any(|re| re.is_match(contents)))
            })
            .cloned()
            .collect()
    }

    /// Uses simple checks to find one or more matching languages. Checks by shebang, filename,
    /// filepath glob, and extension.
    fn find_simple(path: impl AsRef<Path>, contents: &[u8]) -> Vec<Self> {
        let languages = Self::from_shebang(contents);
        if !languages.is_empty() {
            return languages;
        }
        let languages = Self::from_path_filename(&path);
        if !languages.is_empty() {
            return languages;
        }
        let languages = Self::from_glob(&path);
        if !languages.is_empty() {
            return languages;
        }
        Self::from_path_extension(&path)
    }

    /// Picks the best guess from a file's name and contents.
    ///
    /// When checking heuristics, only the first `read_limit` bytes will be read.
    pub fn pick(path: impl AsRef<Path>, contents: &[u8], read_limit: usize) -> Option<Self> {
        let languages = Self::find_simple(&path, contents);
        if languages.len() == 1 {
            return Some(languages[0]);
        }

        let contents = if contents.len() > read_limit {
            &contents[..read_limit]
        } else {
            contents
        };
        let heuristic_contents = std::str::from_utf8(contents).unwrap_or_default();
        let by_heuristics = Self::filter_by_heuristics(&languages, heuristic_contents);

        let found_languages = match by_heuristics.len() {
            0 => languages,
            1 => return Some(by_heuristics[0]),
            _ => by_heuristics,
        };

        found_languages.into_iter().max_by_key(Self::priority)
    }

    /// Returns an object that implements `serde::Serialize` for the language to
    /// serialize the language's attributes. This effectively turns the language
    /// from an `enum` into a `struct`.
    const fn serialize(&self) -> Serialize {
        Serialize {
            name: self.name(),
            category: self.category(),
            color: self.color(),
            nerd_font_glyph: self.nerd_font_glyph(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

impl FromStr for Language {
    type Err = ParseError;

    /// Converts a string of the variant's name into that variant.
    /// This can be useful for setting up language overrides.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_variant(s).ok_or(ParseError)
    }
}

impl serde::Serialize for Language {
    /// Serializes the language into a string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // NOTE A bit redundant LOL
        Self::serialize(self).serialize(serializer)
    }
}

#[cfg(feature = "chromaterm")]
impl Language {
    /// Converts the color to RGB true color.
    pub const fn chromaterm_color(&self) -> chromaterm::colors::True {
        let (r, g, b) = self.color_rgb();
        chromaterm::colors::True::from_rgb(r, g, b)
    }
}

#[cfg(feature = "owo-colors")]
impl Language {
    /// Converts the color to RGB.
    pub const fn owo_color(&self) -> owo_colors::Rgb {
        let (r, g, b) = self.color_rgb();
        owo_colors::Rgb(r, g, b)
    }
}

/// A category for a language.
#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, Eq, Hash, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Data files. Examples: JSON, YAML, XML, CSV, etc.
    Data,
    /// Markup languages. Examples: HTML, Pug, etc.
    Markup,
    /// Languages that define text patterns. Examples: Regex, ABNF, etc.
    Pattern,
    /// Programming languages. Examples: Rust, C, C++, Java, etc.
    Programming,
    /// Prose. Examples: Plain text, Markdown, etc.
    Prose,
    /// Query languages. Examples: SQL, GraphQL, etc.
    Query,
}

/// Helper struct for serializing the attributes of a `Language`.
#[derive(Debug, serde::Serialize)]
struct Serialize {
    name: &'static str,
    category: Category,
    color: &'static str,
    nerd_font_glyph: Option<&'static str>,
}

#[cfg(test)]
mod language_tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        shebang,
        language,
        case::simple(b"#!/bin/sh", Language::Shell),
        case::unix_newline(b"#!/bin/sh\n", Language::Shell),
        case::windows_newline(b"#!/bin/sh\r\n", Language::Shell),
        case::with_env(b"#!/usr/bin/env sh\r\n", Language::Shell)
    )]
    fn test_from_shebang(shebang: &[u8], language: Language) {
        let languages = Language::from_shebang(shebang);
        assert!(languages.contains(&language));
    }
}
