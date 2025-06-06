use super::Gengo;
use super::binary::Binary;
use super::documentation::Documentation;
use super::generated::Generated;
use super::vendored::Vendored;

use crate::file_source::FileSource;
use std::error::Error as ErrorTrait;

/// Builds a new `Gengo` instance.
pub struct Builder<FS: for<'fs> FileSource<'fs>> {
    file_source: FS,
    read_limit: Option<usize>,
}

impl<FS: for<'fs> FileSource<'fs>> Builder<FS> {
    pub const DEFAULT_READ_LIMIT: usize = 1 << 20;

    pub fn new(file_source: FS) -> Self {
        Self {
            file_source,
            read_limit: None,
        }
    }

    /// Sets the limit for how many bytes should be read from each file for
    /// heuristic analysis. If this is not set, `DEFAULT_READ_LIMIT` will be
    /// used.
    pub fn read_limit(mut self, read_limit: usize) -> Self {
        self.read_limit = Some(read_limit);
        self
    }

    pub fn build(self) -> Result<Gengo<FS>, Box<dyn ErrorTrait>> {
        let file_source = self.file_source;
        let read_limit = self.read_limit.unwrap_or(Self::DEFAULT_READ_LIMIT);
        let binary = Binary::new(read_limit);
        let documentation = Documentation::new();
        let generated = Generated::new();
        let vendored = Vendored::new();
        Ok(Gengo {
            file_source,
            read_limit,
            binary,
            documentation,
            generated,
            vendored,
        })
    }
}
