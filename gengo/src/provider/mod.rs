//! Provides file providers.
use crate::Language;
use std::path::Path;

/// Provides files and overrides.
pub trait Provider<'contents, P: AsRef<Path>> {
    type Iter: Iterator<Item = (P, &'contents [u8])>;

    /// Returns an iterator over the files.
    fn files(&self) -> Self::Iter;

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
