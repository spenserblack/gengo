use std::path::Path;

/// A trait for a source of files.
pub trait FileSource<'bytes, P: AsRef<Path>> {
    type Iter: Iterator<Item = (P, &'bytes [u8])>;
    /// Opens the file source from the given directory.
    fn open<DIR: AsRef<Path>>(path: DIR) -> Self;
    /// Returns an iterator over the files in the source.
    fn iter(&self) -> Self::Iter;
}
