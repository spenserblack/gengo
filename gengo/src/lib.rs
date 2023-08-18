pub use builder::Builder;
use documentation::Documentation;
use generated::Generated;
use git2::{AttrCheckFlags, AttrValue, Blob, Commit, ObjectType, Repository, Tree};
use indexmap::IndexMap;
pub use languages::analyzer::Analyzers;
use languages::Category;
pub use languages::Language;
use std::error::Error;
use std::path::{Path, PathBuf};
use vendored::Vendored;

mod builder;
mod documentation;
mod generated;
pub mod languages;
mod vendored;

/// The main entry point for Gengo.
pub struct Gengo {
    repository: Repository,
    analyzers: Analyzers,
    read_limit: usize,
    documentation: Documentation,
    generated: Generated,
    vendored: Vendored,
}

// TODO parse .gitattributes to get language overrides.
impl Gengo {
    const ATTR_CHECK_FLAGS: [AttrCheckFlags; 2] =
        [AttrCheckFlags::NO_SYSTEM, AttrCheckFlags::INDEX_THEN_FILE];
    /// Resolves a revision to a commit.
    fn rev(&self, rev: &str) -> Result<Commit, Box<dyn Error>> {
        let reference = self.repository.revparse_single(rev)?;
        let commit = reference.peel_to_commit()?;
        Ok(commit)
    }

    /// Analyzes each file in the repository at the given revision.
    pub fn analyze(&self, rev: &str) -> Result<IndexMap<PathBuf, Entry>, Box<dyn Error>> {
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
        results: &mut IndexMap<PathBuf, Entry>,
    ) -> Result<(), Box<dyn Error>> {
        for entry in tree.iter() {
            let object = entry.to_object(&self.repository)?;
            match entry.kind() {
                Some(ObjectType::Tree) => {
                    let path = entry.name().ok_or("invalid path")?;
                    let tree = object.as_tree().expect("object to be a tree");
                    let path = Path::new(root).join(path);
                    let path = path.to_str().ok_or("invalid path")?;

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

    fn analyze_blob<P: AsRef<Path>>(
        &self,
        filepath: P,
        blob: &Blob,
        results: &mut IndexMap<PathBuf, Entry>,
    ) -> Result<(), Box<dyn Error>> {
        let contents = blob.content();

        let lang_override = self
            .get_str_attr(&filepath, "gengo-language")?
            .map(|s| s.replace('-', " "))
            .and_then(|s| self.analyzers.get(&s));

        let language =
            lang_override.or_else(|| self.analyzers.pick(&filepath, contents, self.read_limit));

        let language = if let Some(language) = language {
            language.clone()
        } else {
            return Ok(());
        };

        let size = contents.len();
        let generated = self
            .get_boolean_attr(&filepath, "gengo-generated")?
            .unwrap_or_else(|| self.is_generated(&filepath, contents));
        let documentation = self
            .get_boolean_attr(&filepath, "gengo-documentation")?
            .unwrap_or_else(|| self.is_documentation(&filepath, contents));
        let vendored = self
            .get_boolean_attr(&filepath, "gengo-vendored")?
            .unwrap_or_else(|| self.is_vendored(&filepath, contents));

        let detectable = match language.category() {
            Category::Data | Category::Prose => false,
            Category::Programming | Category::Markup | Category::Query => {
                !(generated || documentation || vendored)
            }
        };
        let detectable = self
            .get_boolean_attr(&filepath, "gengo-detectable")?
            .unwrap_or(detectable);

        let path_buf = filepath.as_ref().to_path_buf();
        let entry = Entry {
            language,
            size,
            detectable,
            generated,
            documentation,
            vendored,
        };

        results.insert(path_buf, entry);

        Ok(())
    }

    /// Guesses if a file is generated.
    pub fn is_generated<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.generated.is_generated(filepath, contents)
    }

    /// Guesses if a file is documentation.
    pub fn is_documentation<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.documentation.is_documentation(filepath, contents)
    }

    /// Guesses if a file is vendored.
    pub fn is_vendored<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.vendored.is_vendored(filepath, contents)
    }

    fn get_attr<P: AsRef<Path>>(&self, path: P, attr: &str) -> Result<AttrValue, Box<dyn Error>> {
        let flags = Self::ATTR_CHECK_FLAGS
            .into_iter()
            .reduce(|a, b| a | b)
            .unwrap();
        let attr = self.repository.get_attr(path.as_ref(), attr, flags)?;
        let attr = AttrValue::from_string(attr);
        Ok(attr)
    }

    fn get_boolean_attr<P: AsRef<Path>>(
        &self,
        path: P,
        attr: &str,
    ) -> Result<Option<bool>, Box<dyn Error>> {
        let attr = self.get_attr(path, attr)?;
        let attr = match attr {
            AttrValue::True => Some(true),
            AttrValue::False => Some(false),
            AttrValue::Unspecified => None,
            // NOTE To avoid being overly strict, we'll just ignore invalid attributes.
            _ => None,
        };
        Ok(attr)
    }

    fn get_str_attr<P: AsRef<Path>>(
        &self,
        path: P,
        attr: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let attr = self.get_attr(path, attr)?;
        let attr = match attr {
            AttrValue::String(s) => Some(s),
            AttrValue::Unspecified => None,
            // NOTE To avoid being overly strict, we'll just ignore invalid attributes.
            _ => None,
        };
        Ok(attr.map(String::from))
    }
}

/// A single entry in the language statistics.
#[derive(Debug)]
pub struct Entry {
    /// The detected language.
    language: Language,
    /// The size of the file.
    size: usize,
    /// If the file is detectable (should not be ignored).
    detectable: bool,
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

    /// If the file is detectable (should not be ignored).
    pub fn detectable(&self) -> bool {
        self.detectable
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
