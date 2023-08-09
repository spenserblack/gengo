pub use builder::Builder;
use git2::{Blob, Commit, ObjectType, Repository, Tree};
use indexmap::IndexMap;
pub use languages::analyzer::Analyzers;
pub use languages::Language;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;

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
        let mut results = IndexMap::new();
        let commit = self.rev(rev)?;
        let tree = commit.tree()?;
        self.analyze_tree("", &tree, &mut results)?;
        Ok(results)
    }

    fn analyze_tree(
        &self,
        root: &str,
        tree: &Tree,
        results: &mut IndexMap<String, Entry>,
    ) -> Result<(), Box<dyn Error>> {
        for entry in tree.iter() {
            let object = entry.to_object(&self.repository)?;
            match entry.kind() {
                Some(ObjectType::Tree) => {
                    let path = entry.name().ok_or("invalid path")?;
                    let tree = object.as_tree().expect("object to be a tree");

                    self.analyze_tree(path, tree, results)?;
                }
                Some(ObjectType::Blob) => {
                    let path = entry.name().ok_or("invalid path").unwrap();
                    let filepath = Path::new(root).join(path);
                    let filepath = filepath.as_os_str();
                    let blob = object.as_blob().expect("object to be a blob");

                    self.analyze_blob(filepath, blob, results)?;
                }
                _ => continue,
            }
        }
        Ok(())
    }

    fn analyze_blob(
        &self,
        filepath: &OsStr,
        blob: &Blob,
        results: &mut IndexMap<String, Entry>,
    ) -> Result<(), Box<dyn Error>> {
        let contents = blob.content();
        let language = self.analyzers.pick(filepath, contents, self.read_limit);
        let language = if let Some(language) = language {
            language.clone()
        } else {
            return Ok(());
        };

        let size = contents.len();
        let generated = self.is_generated(filepath, contents);
        let documentation = self.is_documentation(filepath, contents);
        let vendored = self.is_vendored(filepath, contents);

        let path = String::from(filepath.to_str().ok_or("invalid path")?);
        let entry = Entry {
            language,
            size,
            generated,
            documentation,
            vendored,
        };

        results.insert(path, entry);

        Ok(())
    }

    /// Guesses if a file is generated.
    pub fn is_generated(&self, filepath: &OsStr, _contents: &[u8]) -> bool {
        filepath.to_str().unwrap_or_default().starts_with("dist/")
    }

    /// Guesses if a file is documentation.
    pub fn is_documentation(&self, filepath: &OsStr, _contents: &[u8]) -> bool {
        filepath.to_str().unwrap_or_default().starts_with("docs/")
    }

    /// Guesses if a file is vendored.
    pub fn is_vendored(&self, filepath: &OsStr, _contents: &[u8]) -> bool {
        filepath.to_str().unwrap_or_default().starts_with("node_modules/")
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
