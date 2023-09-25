use super::documentation::Documentation;
use super::generated::Generated;
use super::vendored::Vendored;
use super::Analyzers;
use super::Gengo;
use super::{Error, ErrorKind};
use gix::discover::Error as DiscoverError;
use std::error::Error as ErrorTrait;
use std::path::Path;
use crate::file_source::Git;

/// Builds a new `Gengo` instance.
///
/// # Example
///
/// ```no_run
/// use gengo::Builder;
/// let gengo = Builder::new("path/to/repo").build().unwrap();
/// ```
pub struct Builder<P: AsRef<Path>> {
    repository_path: P,
    rev: Option<String>,
    analyzers: Option<Analyzers>,
    read_limit: Option<usize>,
}

impl<P: AsRef<Path>> Builder<P> {
    pub const DEFAULT_READ_LIMIT: usize = 1 << 20;

    pub fn new(repository_path: P) -> Self {
        Self {
            repository_path,
            rev: None,
            analyzers: None,
            read_limit: None,
        }
    }

    /// Sets the `Analyzers` to use. If this is not set,
    /// `Analyzers::default()` will be used.
    pub fn analyzers(mut self, analyzers: Analyzers) -> Self {
        self.analyzers = Some(analyzers);
        self
    }

    /// Sets the limit for how many bytes should be read from each file for
    /// heuristic analysis. If this is not set, `DEFAULT_READ_LIMIT` will be
    /// used.
    pub fn read_limit(mut self, read_limit: usize) -> Self {
        self.read_limit = Some(read_limit);
        self
    }

    pub fn build<'repo>(self) -> Result<Gengo<'repo, Git>, Box<dyn ErrorTrait>> {
        let file_source = Git::new(self.repository_path, self.rev.map(|s| s.as_str()).unwrap_or("HEAD"))?;
        let analyzers = self.analyzers.unwrap_or_default();
        let read_limit = self.read_limit.unwrap_or(Self::DEFAULT_READ_LIMIT);
        let documentation = Documentation::new();
        let generated = Generated::new();
        let vendored = Vendored::new();
        Ok(Gengo {
            file_source,
            analyzers,
            read_limit,
            documentation,
            generated,
            vendored,
        })
    }
}
