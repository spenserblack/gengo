//! Gengo is a language detection library for Git repositories.
//! While it is possible to provide your own definitions for
//! language detection, Gengo comes with a set of built-in
//! definitions.
//!
//! # Built-in Languages
#![doc = include_str!(concat!(env!("OUT_DIR"), "/language-list.md"))]
pub use analysis::Analysis;
pub use analysis::Iter as AnalysisIter;
pub use builder::Builder;
use documentation::Documentation;
pub use error::{Error, ErrorKind};
use generated::Generated;
use gix::attrs::StateRef;
use gix::bstr::ByteSlice;
use gix::object::tree::EntryMode;
use gix::prelude::FindExt;
use glob::MatchOptions;
use indexmap::IndexMap;
pub use languages::analyzer::Analyzers;
use languages::Category;
pub use languages::Language;
use std::path::{Path, PathBuf};
use vendored::Vendored;

pub mod analysis;
mod builder;
mod documentation;
mod error;
mod generated;
pub mod languages;
mod vendored;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

/// Shared match options for consistent behavior.
const GLOB_MATCH_OPTIONS: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

/// The main entry point for Gengo.
pub struct Gengo {
    repository: gix::Repository,
    analyzers: Analyzers,
    read_limit: usize,
    documentation: Documentation,
    generated: Generated,
    vendored: Vendored,
}

#[derive(Clone)]
struct GitState {
    attr_stack: gix::worktree::Stack,
    attr_matches: gix::attrs::search::Outcome,
    tree_id: gix::ObjectId,
}

impl GitState {
    fn new(repo: &gix::Repository, rev: &str) -> Result<Self> {
        let tree_id = repo.rev_parse_single(rev)?.object()?.peel_to_tree()?.id;
        let index = repo.index_from_tree(&tree_id)?;
        let attr_stack = repo.attributes_only(
            &index,
            gix::worktree::stack::state::attributes::Source::IdMappingThenWorktree,
        )?;
        let attr_matches = attr_stack.selected_attribute_matches([
            "gengo-language",
            "gengo-generated",
            "gengo-documentation",
            "gengo-vendored",
            "gengo-detectable",
        ]);
        Ok(Self {
            attr_stack,
            attr_matches,
            tree_id,
        })
    }
}

impl Gengo {
    /// Analyzes each file in the repository at the given revision.
    pub fn analyze(&self, rev: &str) -> Result<Analysis> {
        let mut results = IndexMap::new();
        let mut state = GitState::new(&self.repository, rev)?;
        self.analyze_tree("", state.tree_id, &mut state, &mut results)?;
        Ok(Analysis(results))
    }

    fn analyze_tree(
        &self,
        root: &str,
        tree: gix::ObjectId,
        state: &mut GitState,
        results: &mut IndexMap<PathBuf, Entry>,
    ) -> Result<()> {
        let tree = self.repository.find_object(tree)?.into_tree();
        for entry in tree.iter() {
            let entry = entry?;
            match entry.mode() {
                EntryMode::Tree => {
                    let path = Path::new(root).join(entry.filename().to_str_lossy().as_ref());
                    let path = path
                        .to_str()
                        .expect("created by lossy conversion - cannot fail");

                    self.analyze_tree(path, entry.object_id(), state, results)?;
                }
                EntryMode::Blob | EntryMode::BlobExecutable => {
                    let path = Path::new(root).join(entry.filename().to_str_lossy().as_ref());
                    let path = path
                        .to_str()
                        .expect("created by lossy conversion - cannot fail");

                    self.analyze_blob(path, entry.oid(), state, results)?;
                }
                EntryMode::Link => {}
                EntryMode::Commit => {
                    // TODO: recurse submodules
                    continue;
                }
            }
        }
        Ok(())
    }

    fn analyze_blob(
        &self,
        filepath: impl AsRef<Path>,
        blob: &gix::oid,
        state: &mut GitState,
        results: &mut IndexMap<PathBuf, Entry>,
    ) -> Result<()> {
        let filepath = filepath.as_ref();
        let blob = self.repository.find_object(blob)?;
        let contents = blob.data.as_slice();
        state
            .attr_stack
            .at_path(filepath, Some(false), |id, buf| {
                self.repository.objects.find_blob(id, buf)
            })?
            .matching_attributes(&mut state.attr_matches);

        let mut attrs = [None, None, None, None, None];
        state
            .attr_matches
            .iter_selected()
            .zip(attrs.iter_mut())
            .for_each(|(info, slot)| {
                *slot =
                    (info.assignment.state != gix::attrs::StateRef::Unspecified).then_some(info);
            });

        let lang_override = attrs[0]
            .as_ref()
            .and_then(|info| match info.assignment.state {
                StateRef::Value(v) => v.as_bstr().to_str().ok().map(|s| s.replace('-', " ")),
                _ => None,
            })
            .and_then(|s| self.analyzers.get(&s));

        let language =
            lang_override.or_else(|| self.analyzers.pick(filepath, contents, self.read_limit));

        let language = if let Some(language) = language {
            language
        } else {
            return Ok(());
        };

        let generated = attrs[1]
            .as_ref()
            .map(|info| info.assignment.state.is_set())
            .unwrap_or_else(|| self.is_generated(filepath, contents));
        let documentation = attrs[2]
            .as_ref()
            .map(|info| info.assignment.state.is_set())
            .unwrap_or_else(|| self.is_documentation(filepath, contents));
        let vendored = attrs[3]
            .as_ref()
            .map(|info| info.assignment.state.is_set())
            .unwrap_or_else(|| self.is_vendored(filepath, contents));

        let detectable = match language.category() {
            Category::Data | Category::Prose => false,
            Category::Programming | Category::Markup | Category::Query => {
                !(generated || documentation || vendored)
            }
        };
        let detectable = attrs[4]
            .as_ref()
            .map(|info| info.assignment.state.is_set())
            .unwrap_or(detectable);

        let size = contents.len();
        let entry = Entry {
            language: language.clone(),
            size,
            detectable,
            generated,
            documentation,
            vendored,
        };
        results.insert(filepath.to_path_buf(), entry);
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
