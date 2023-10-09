//! Provides sources to get files and their attributes.

pub use git::Git;
use std::marker::{Send, Sync};
use std::path::Path;

mod git;

/// Provides files and overrides.
pub trait FileSource<'files>: Sync {
    type Filepath: AsRef<Path>;
    type Contents: AsRef<[u8]>;
    type Entry: Send;
    type Iter: Iterator<Item = Self::Entry> + Send;

    /// Returns an iterator over the entries use to get filenames and contents.
    fn entries(&'files self) -> crate::Result<Self::Iter>;

    /// Gets a filename from an entry.
    fn filepath(&'files self, entry: &Self::Entry) -> crate::Result<Self::Filepath>;

    /// Gets file contents from an entry.
    fn contents(&'files self, entry: &Self::Entry) -> crate::Result<Self::Contents>;

    /// Provides combined overrides for the file.
    fn overrides<O: AsRef<Path>>(&self, path: O) -> Overrides {
        Overrides {
            language: self.language_override(&path),
            is_documentation: self.is_documentation_override(&path),
            is_generated: self.is_generated_override(&path),
            is_vendored: self.is_vendored_override(&path),
            is_detectable: self.is_detectable_override(&path),
        }
    }

    /// Provides an optional override for the detected language.
    fn language_override<O: AsRef<Path>>(&self, _path: O) -> Option<String> {
        None
    }

    /// Provides an optional override for documentation file detection.
    fn is_documentation_override<O: AsRef<Path>>(&self, _path: O) -> Option<bool> {
        None
    }

    /// Provides an optional override for generated file detection.
    fn is_generated_override<O: AsRef<Path>>(&self, _path: O) -> Option<bool> {
        None
    }

    /// Provides an optional override for vendored file detection.
    fn is_vendored_override<O: AsRef<Path>>(&self, _path: O) -> Option<bool> {
        None
    }

    /// Provides an optional override for if the file is detectable.
    fn is_detectable_override<O: AsRef<Path>>(&self, _path: O) -> Option<bool> {
        None
    }
}

#[non_exhaustive]
#[derive(Default)]
pub struct Overrides {
    pub language: Option<String>,
    pub is_documentation: Option<bool>,
    pub is_generated: Option<bool>,
    pub is_vendored: Option<bool>,
    pub is_detectable: Option<bool>,
}
