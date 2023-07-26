/// Analyzes a language.
use super::{Category, Language};

pub struct Analyzers(Vec<Analyzer>);
struct Analyzer {}

impl Analyzer {
    /// Create a new language analyzer.
    fn new(
        language: Language,
        category: Category,
        color: &str,
        extensions: &[&str],
        filenames: &[&str],
        patterns: &[&str],
        heuristics: &[&str],
        priority: f32,
    ) -> Self {
        todo!()
    }
}

impl Default for Analyzers {
    /// Create a new language analyzer with default values.
    fn default() -> Self {
        Self(include!(concat!(env!("OUT_DIR"), "/analyzer.rs")))
    }
}
