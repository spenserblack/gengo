//! Gengo is a language detection library for collections of files.
//! Currently, it supports reading from git repositories.
//!
//! # Features
//!
//! ## `directory`
//!
//! Provides the `Directory` file source, which reads files from a local directory.
//!
//! ## `git`
//!
//! Provides the `Git` file source, which reads files from a git repository. It reads
//! from a specified revision and supports git attributes for overrides, making it the
//! most similar to [GitHub Linguist][github-linguist]. Because of this, it also works
//! on bare repositories.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "git")]
//! # {
//! use gengo::{Builder, Git};
//! let git = Git::new("path/to/repo", "HEAD").unwrap();
//! let gengo = Builder::new(git).build().unwrap();
//! let results = gengo.analyze().unwrap();
//! # }
//! ```
//!
//! [github-linguist]: https://github.com/github-linguist/linguist

pub use analysis::Analysis;
use binary::Binary;
pub use builder::Builder;
use documentation::Documentation;

pub use error::{Error, ErrorKind};
use generated::Generated;

#[cfg(feature = "directory")]
pub use file_source::Directory;

#[cfg(feature = "git")]
pub use file_source::Git;

pub use file_source::FileSource;
use glob::MatchOptions;
use indexmap::IndexMap;
use language::Category;
pub use language::Language;

use std::error::Error as ErrorTrait;
use std::path::Path;

use vendored::Vendored;

use rayon::prelude::{FromParallelIterator, ParallelBridge, ParallelIterator};
use serde::Serialize;

pub mod analysis;
mod binary;
mod builder;
mod documentation;
mod error;
mod file_source;
mod generated;
pub mod language;
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
    read_limit: usize,
    binary: Binary,
    documentation: Documentation,
    generated: Generated,
    vendored: Vendored,
}

impl<FS: for<'fs> FileSource<'fs>> Gengo<FS> {
    /// Analyzes each file in the repository at the given revision.
    pub fn analyze(&self) -> Result<Analysis> {
        let state = self.file_source.state()?;
        let entries = self
            .file_source
            .entries()?
            .par_bridge()
            .map_with(state, |state, entry| {
                let filepath = self.file_source.filepath(&entry, state).ok()?;
                let contents = self.file_source.contents(&entry, state).ok()?;

                let entry = self.analyze_blob(&filepath, contents, state)?;
                Some((filepath.as_ref().to_owned(), entry))
            })
            .filter_map(|entry| entry);
        let entries = IndexMap::from_par_iter(entries);

        Ok(Analysis(entries))
    }

    fn analyze_blob(
        &self,
        filepath: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
        state: &mut <FS as FileSource>::State,
    ) -> Option<Entry> {
        let overrides = self.file_source.overrides(&filepath, state);
        let filepath = filepath.as_ref();
        let contents = contents.as_ref();

        // NOTE Users might be surprised if there is an override for a binary file but it
        //      is still skipped, but this should be a rare case.
        if self.is_binary(filepath, contents) {
            return None;
        }

        let language = overrides
            .language
            .or_else(|| Language::pick(filepath, contents, self.read_limit))?;
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
            Category::Pattern | Category::Programming | Category::Markup | Category::Query => {
                !(generated || documentation || vendored)
            }
        };
        let detectable = overrides.is_detectable.unwrap_or(detectable);

        let size = contents.len();
        let entry = Entry {
            language,
            size,
            detectable,
            generated,
            documentation,
            vendored,
        };
        Some(entry)
    }

    /// Guesses if a file is generated.
    pub fn is_generated(&self, filepath: impl AsRef<Path>, contents: &[u8]) -> bool {
        self.generated.is_generated(filepath, contents)
    }

    /// Guesses if a file is documentation.
    pub fn is_documentation(&self, filepath: impl AsRef<Path>, contents: &[u8]) -> bool {
        self.documentation.is_documentation(filepath, contents)
    }

    /// Guesses if a file is vendored.
    pub fn is_vendored(&self, filepath: impl AsRef<Path>, contents: &[u8]) -> bool {
        self.vendored.is_vendored(filepath, contents)
    }

    /// Guesses if a file is binary.
    pub fn is_binary(&self, filepath: impl AsRef<Path>, contents: &[u8]) -> bool {
        self.binary.is_binary(filepath, contents)
    }
}

/// A single entry in the language statistics.
#[derive(Debug, Serialize)]
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
