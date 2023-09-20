//! Provides sources to get files and their attributes.
use crate::Language;
use std::path::Path;
use std::result::Result as BaseResult;
pub use git::Git;
use crate::GenericError;

type Result<T, E = GenericError> = BaseResult<T, E>;

mod git;

/// Provides files and overrides.
pub trait FileSource<'contents> {
    type Filepath: AsRef<Path> + Send + Sync;
    type Contents: AsRef<[u8]> + Send + Sync;
    type Iter: Iterator<Item = (Self::Filepath, Self::Contents)>;

    /// Returns an iterator over the files.
    fn files(&self) -> Result<Self::Iter>;

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

    /// Provides an optional override for the detected language.
    fn language_override<O: AsRef<Path>>(&self, _path: O) -> Option<Language> {
        None
    }
}
