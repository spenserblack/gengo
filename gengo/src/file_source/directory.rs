use super::FileSource;
use crate::Result;
use ignore::{Walk, WalkBuilder};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// A file source that reads files from a directory.
///
/// Will try to ignore files be default. See [`WalkBuilder`](ignore::WalkBuilder)
/// for more information.
pub struct Directory {
    buf_size: usize,
    walk_builder: WalkBuilder,
}

pub struct Iter {
    walk: Walk,
}

// TODO Simpler API than having to copy the same read_limit/buf_size value to multiple places?
impl Directory {
    /// Creates a new directory file source.
    ///
    /// Set `buf_size` to a reasonable value for your system. You most likely
    /// would want to set this to the same value as `read_limit` when building
    /// a [`Gengo`](crate::Gengo) instance with a [`Builder`](crate::Builder).
    pub fn new<P: AsRef<Path>>(path: P, buf_size: usize) -> Result<Self> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err("path is not a directory".into());
        }
        let walk_builder = WalkBuilder::new(path);
        let directory = Self {
            buf_size,
            walk_builder,
        };
        Ok(directory)
    }
}

impl<'files> FileSource<'files> for Directory {
    type Entry = PathBuf;
    type Filepath = PathBuf;
    type Contents = Vec<u8>;
    type State = ();
    type Iter = Iter;

    fn entries(&'files self) -> crate::Result<Self::Iter> {
        // NOTE `new` should assert that `path` is always a directory
        let walk = self.walk_builder.build();
        Ok(Iter { walk })
    }

    fn filepath(
        &'files self,
        entry: &Self::Entry,
        _state: &mut Self::State,
    ) -> crate::Result<Self::Filepath> {
        Ok(entry.to_owned())
    }

    fn contents(
        &'files self,
        entry: &Self::Entry,
        _state: &mut Self::State,
    ) -> crate::Result<Self::Contents> {
        let mut reader = File::open(entry)?.take(self.buf_size.try_into()?);
        let mut buf = vec![0; self.buf_size];
        let read_amount = reader.read(&mut buf)?;
        buf.resize(read_amount, 0);
        Ok(buf)
    }

    fn state(&'files self) -> crate::Result<Self::State> {
        Ok(())
    }
}

impl Iterator for Iter {
    /// Deeply iterates over the directory.
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO Handle error?
        self.walk
            .next()
            .and_then(|entry| entry.ok())
            .map(|dir_entry| dir_entry.path().to_owned())
    }
}
