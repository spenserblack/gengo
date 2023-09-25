//! Provides sources to get files and their attributes.
use crate::Language;
use std::path::Path;
use std::result::Result as BaseResult;
pub use git::Git;
use crate::GenericError;

type Result<T, E = GenericError> = BaseResult<T, E>;

mod git;

/// Provides files and overrides.
pub trait FileSource<'files> {
    type Filepath: AsRef<Path> + Send + Sync;
    type Contents: AsRef<[u8]> + Send + Sync;
    type Iter: Iterator<Item = (Self::Filepath, Self::Contents)>;

    /// Returns an iterator over the files.
    fn files(&'files self) -> Result<Self::Iter>;

    /// Provides combined overrides for the file.
    fn overrides<O: AsRef<Path>>(
        &self,
        path: O,
    ) -> Overrides {
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
