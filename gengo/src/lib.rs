pub use builder::Builder;
use git2::Commit;
use git2::Repository;
use indexmap::IndexMap;
pub use languages::analyzer::Analyzers;
pub use languages::Language;
use std::error::Error;
use std::ffi::OsStr;

mod builder;
pub mod languages;

/// The main entry point for Gengo.
pub struct Gengo {
    repository: Repository,
    analyzers: Analyzers,
    read_limit: usize,
}

// TODO parse .gitattributes to get language overrides.
impl Gengo {
    /// Resolves a revision to a commit.
    fn rev(&self, rev: &str) -> Result<Commit, Box<dyn Error>> {
        let reference = self.repository.revparse_single(rev)?;
        let commit = reference.peel_to_commit()?;
        Ok(commit)
    }

    /// Analyzes each file in the repository at the given revision.
    pub fn analyze(&self, rev: &str) -> Result<IndexMap<String, Entry>, Box<dyn Error>> {
        let commit = self.rev(rev)?;
        let tree = commit.tree()?;
        let mut results = IndexMap::new();
        for entry in tree.iter() {
            let path = entry.name().ok_or("invalid path")?;
            let filepath = OsStr::new(path);
            // TODO Skip anything that is likely binary
            let blob = self.repository.find_blob(entry.id())?;
            let contents = blob.content();

            let language = self.analyzers.pick(filepath, contents, self.read_limit);
            let language = if let Some(language) = language {
                language.clone()
            } else {
                continue;
            };

            let size = contents.len();
            let generated = self.is_generated(filepath, contents);
            let documentation = self.is_documentation(filepath, contents);
            let vendored = self.is_vendored(filepath, contents);

            let entry = Entry {
                language,
                size,
                generated,
                documentation,
                vendored,
            };

            let path = String::from(path);
            results.insert(path, entry);
        }
        Ok(results)
    }

    /// Guesses if a file is generated.
    pub fn is_generated(&self, _filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }

    /// Guesses if a file is documentation.
    pub fn is_documentation(&self, _filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }

    /// Guesses if a file is vendored.
    pub fn is_vendored(&self, _filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }
}

/// A single entry in the language statistics.
#[derive(Debug)]
pub struct Entry {
    /// The detected language.
    language: Language,
    /// The size of the file.
    size: usize,
    /// If the file was generated.
    generated: bool,
    /// If the file is documentation.
    documentation: bool,
    /// If the file is vendored.
    vendored: bool,
}

impl Entry {
    /// The detected language.
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// The size of the file.
    pub fn size(&self) -> usize {
        self.size
    }

    /// If the file was generated.
    pub fn generated(&self) -> bool {
        self.generated
    }

    /// If the file is documentation.
    pub fn documentation(&self) -> bool {
        self.documentation
    }

    /// If the file is vendored.
    pub fn vendored(&self) -> bool {
        self.vendored
    }
}
