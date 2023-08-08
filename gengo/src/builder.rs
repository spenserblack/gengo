
use super::Gengo;
use super::Analyzers;
use std::path::Path;
use git2::Repository;
use std::error::Error;

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
    analyzers: Option<Analyzers>,
}

impl<P: AsRef<Path>> Builder<P> {
    pub fn new(repository_path: P) -> Self {
        Self {
            repository_path,
            analyzers: None,
        }
    }

    /// Sets the `Analyzers` to use. If this is not set,
    /// `Analyzers::default()` will be used.
    pub fn analyzers(mut self, analyzers: Analyzers) -> Self {
        self.analyzers = Some(analyzers);
        self
    }

    pub fn build(self) -> Result<Gengo, Box<dyn Error>> {
        let repository = Repository::discover(self.repository_path)?;
        let analyzers = self.analyzers.unwrap_or_default();
        Ok(Gengo { repository, analyzers })
    }
}
