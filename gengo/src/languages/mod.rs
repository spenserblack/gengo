#[cfg(feature = "owo-colors")]
use owo_colors::Rgb;
use serde::{Deserialize, Serialize};
#[cfg(feature = "owo-colors")]
use std::error::Error;
pub mod analyzer;
mod matcher;

include!(concat!(env!("OUT_DIR"), "/languages_generated.rs"));

const LANGUAGE_DEFINITIONS: &str = include_str!(concat!(env!("OUT_DIR"), "/languages.json"));

/// A programming language.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[deprecated]
pub struct LanguageOld {
    name: String,
    category: Category,
    color: String,
}

impl LanguageOld {
    /// Returns the name of the language.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the category of the language.
    pub fn category(&self) -> &Category {
        &self.category
    }

    /// Returns the color of the language.
    pub fn color(&self) -> &str {
        &self.color
    }

    /// Tries to convert the color to RGB.
    #[cfg(feature = "owo-colors")]
    pub fn owo_color(&self) -> Result<Rgb, Box<dyn Error>> {
        let hex_string = self.color.strip_prefix('#').ok_or("Expected '#' prefix")?;
        if hex_string.len() != 6 {
            return Err("Expected 6 characters".into());
        }
        let bytes = u32::from_str_radix(hex_string, 16)?;
        let r = ((bytes >> 16) & 0xFF) as u8;
        let g = ((bytes >> 8) & 0xFF) as u8;
        let b = (bytes & 0xFF) as u8;
        Ok(Rgb(r, g, b))
    }
}

/// A category for a language.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
