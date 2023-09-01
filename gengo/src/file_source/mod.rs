use std::path::Path;
pub use git::Git;

mod git;

/// A trait for a source of files.
pub trait FileSource<'bytes, P: AsRef<Path>> {
    type Error;
    type Iter: Iterator<Item = (P, &'bytes [u8])>;
    /// Opens the file source from the given directory.
    fn open<Dir: AsRef<Path>>(path: Dir) -> Result<Self, Self::Error>
    where
        Self: Sized;
    /// Returns an iterator over the files in the source.
    fn iter(&self) -> Result<Self::Iter, Self::Error>;
}
