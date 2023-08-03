use serde::Deserialize;
pub mod analyzer;

const LANGUAGE_DEFINITIONS: &str = include_str!(concat!(env!("OUT_DIR"), "/languages.json"));

/// A programming language.
#[derive(Clone, Debug, Deserialize)]
pub struct Language {
    name: String,
    category: Category,
    color: String,
}

/// A category for a language.
#[derive(Clone, Debug, Deserialize)]
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
}
