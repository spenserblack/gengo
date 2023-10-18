use super::FileSource;
use crate::Result;
use std::fs::{File, ReadDir};
use std::io::Read;
use std::path::{Path, PathBuf};

/// A file source that reads files from a directory.
pub struct Directory {
    path: PathBuf,
    buf_size: usize,
}

pub struct Iter {
    paths: Vec<ReadDir>,
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
        let path = path.to_owned();
        let directory = Self { path, buf_size };
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
        let paths = vec![self.path.read_dir()?];
        Ok(Iter { paths })
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
        loop {
            let read_dir = self.paths.last_mut()?;
            let entry = match read_dir.next() {
                Some(entry) => entry,
                None => {
                    self.paths.pop();
                    continue;
                }
            };
            // TODO Handle errors?
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                self.paths.push(path.read_dir().ok()?);
                continue;
            } else if !path.is_file() {
                continue;
            }
            break Some(path);
        }
    }
}
