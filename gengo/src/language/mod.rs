use std::str::FromStr;

macro_rules! _include {
    ($path:literal) => {
        include!(concat!(env!("OUT_DIR"), "/languages/", $path));
    };
}

include!(concat!(env!("OUT_DIR"), "/language_generated.rs"));
_include!("language.rs");
_include!("category_mixin.rs");
_include!("name_mixin.rs");
_include!("parse_variant_mixin.rs");
_include!("color_mixin.rs");
_include!("priority_mixin.rs");
_include!("from_extension_mixin.rs");
_include!("from_filename_mixin.rs");
_include!("from_interpreter_mixin.rs");

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

        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^#!(?:/usr(?:/local)?)?/bin/(?:env\s+)?([\w\d]+)\r?$").unwrap()
        });

        RE.captures(first_line)
            .and_then(|c| c.get(1))
            .map_or(vec![], |m| {
                let interpreter = m.as_str();
                Self::from_interpreter(interpreter)
            })
    }

    /// Returns an object that implements `serde::Serialize` for the language to
    /// serialize the language's attributes. This effectively turns the language
    /// from an `enum` into a `struct`.
    const fn serialize(&self) -> Serialize {
        Serialize {
            name: self.name(),
            category: self.category(),
            color: self.color(),
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

#[cfg(feature = "owo-colors")]
impl Language {
    /// Converts the color to RGB.
    pub fn owo_color(&self) -> owo_colors::Rgb {
        let hex_string = self.color().strip_prefix('#').expect("'#' prefix");
        assert_eq!(hex_string.len(), 6, "Expected 6 characters");
        let bytes = u32::from_str_radix(hex_string, 16).expect("valid hex string");
        let r = ((bytes >> 16) & 0xFF) as u8;
        let g = ((bytes >> 8) & 0xFF) as u8;
        let b = (bytes & 0xFF) as u8;
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
