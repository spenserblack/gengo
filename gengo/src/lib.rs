use git2::Repository;
use git2::Commit;
use std::error::Error;
pub use languages::analyzer::Analyzers;
pub use languages::Language;
pub use builder::Builder;
use std::ffi::OsStr;
use indexmap::IndexMap;

pub mod languages;
mod builder;

/// The main entry point for Gengo.
pub struct Gengo {
    repository: Repository,
    analyzers: Analyzers,
    read_limit: usize,
}

impl Gengo {
    /// Resolves a revision to a commit.
    fn rev(&self, rev: &str) -> Result<Commit, Box<dyn Error>> {
        let reference = self.repository.revparse_single(rev)?;
        let commit = reference.peel_to_commit()?;
        Ok(commit)
    }

    /// Analyzes each file in the repository at the given revision.
    pub fn analyze(&self, rev: &str) -> Result<IndexMap<String, &Language>, Box<dyn Error>> {
        let commit = self.rev(rev)?;
        let tree = commit.tree()?;
        let mut results = IndexMap::new();
        for entry in tree.iter() {
            let path = entry.name().ok_or("invalid path")?;
            let filepath = OsStr::new(path);
            let blob = self.repository.find_blob(entry.id())?;
            let contents = blob.content();
            let language = self.analyzers.pick(filepath, contents, self.read_limit);
            let language = if let Some(language) = language {
                language
            } else {
                continue;
            };
            let path = String::from(path);
            results.insert(path, language);
        }
        Ok(results)
    }
}
