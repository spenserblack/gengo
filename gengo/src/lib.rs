//! Gengo is a language detection library for Git repositories.
//! While it is possible to provide your own definitions for
//! language detection, Gengo comes with a set of built-in
//! definitions.
//!
//! # Built-in Languages
#![doc = include_str!(concat!(env!("OUT_DIR"), "/language-list.md"))]

pub use analysis::Analysis;
pub use builder::Builder;
use documentation::Documentation;

pub use error::{Error, ErrorKind};
use generated::Generated;

pub use file_source::FileSource;
use glob::MatchOptions;
pub use languages::analyzer::Analyzers;
use languages::Category;
pub use languages::Language;

use std::error::Error as ErrorTrait;
use std::path::Path;

use vendored::Vendored;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub mod analysis;
mod builder;
mod documentation;
mod error;
mod file_source;
mod generated;
pub mod languages;
mod vendored;

type GenericError = Box<dyn ErrorTrait>;
type Result<T, E = GenericError> = std::result::Result<T, E>;

/// Shared match options for consistent behavior.
const GLOB_MATCH_OPTIONS: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

/// The main entry point for Gengo.
pub struct Gengo<FS: for<'fs> FileSource<'fs>> {
    file_source: FS,
    analyzers: Analyzers,
    read_limit: usize,
    documentation: Documentation,
    generated: Generated,
    vendored: Vendored,
}

impl<FS: for<'fs> FileSource<'fs>> Gengo<FS> {
    /// Analyzes each file in the repository at the given revision.
    pub fn analyze(&self) -> Result<Analysis> {
        // TODO Avoid this conversion to Vec?
        let files: Vec<_> = self.file_source.files()?.collect();
        // let mut entries = Vec::with_capacity(files.len());
        let entries: Vec<(_, _)> = files
            .par_iter()
            .filter_map(|(path, contents)| {
                let entry = self.analyze_blob(path, contents)?;
                Some((path.as_ref().to_owned(), entry))
            })
            .collect();

        Ok(Analysis(entries))
    }

    fn analyze_blob(
        &self,
        filepath: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> Option<Entry> {
        let overrides = self.file_source.overrides(&filepath);
        let filepath = filepath.as_ref();
        let contents = contents.as_ref();

        let lang_override = overrides.language.and_then(|s| self.analyzers.get(&s));

        let language =
            lang_override.or_else(|| self.analyzers.pick(filepath, contents, self.read_limit))?;

        let generated = overrides
            .is_generated
            .unwrap_or_else(|| self.is_generated(filepath, contents));
        let documentation = overrides
            .is_documentation
            .unwrap_or_else(|| self.is_documentation(filepath, contents));
        let vendored = overrides
            .is_vendored
            .unwrap_or_else(|| self.is_vendored(filepath, contents));

        let detectable = match language.category() {
            Category::Data | Category::Prose => false,
            Category::Programming | Category::Markup | Category::Query => {
                !(generated || documentation || vendored)
            }
        };
        let detectable = overrides.is_detectable.unwrap_or(detectable);

        let size = contents.len();
        let entry = Entry {
            language: language.clone(),
            size,
            detectable,
            generated,
            documentation,
            vendored,
        };
        Some(entry)
    }

    /// Guesses if a file is generated.
    pub fn is_generated<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.generated.is_generated(filepath, contents)
    }

    /// Guesses if a file is documentation.
    pub fn is_documentation<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.documentation.is_documentation(filepath, contents)
    }

    /// Guesses if a file is vendored.
    pub fn is_vendored<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.vendored.is_vendored(filepath, contents)
    }
}

/// A single entry in the language statistics.
#[derive(Debug)]
pub struct Entry {
    /// The detected language.
    language: Language,
    /// The size of the file.
    size: usize,
    /// If the file is detectable (should not be ignored).
    detectable: bool,
    /// If the file was generated.
    generated: bool,
    /// If the file is documentation.
    documentation: bool,
    /// If the file is vendored.
    vendored: bool,
}

impl Entry {
    /// The detected language.
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// The size of the file.
    pub fn size(&self) -> usize {
        self.size
    }

    /// If the file is detectable (should not be ignored).
    pub fn detectable(&self) -> bool {
        self.detectable
    }

    /// If the file was generated.
    pub fn generated(&self) -> bool {
        self.generated
    }

    /// If the file is documentation.
    pub fn documentation(&self) -> bool {
        self.documentation
    }

    /// If the file is vendored.
    pub fn vendored(&self) -> bool {
        self.vendored
    }
}
