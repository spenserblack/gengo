//! Gengo is a language detection library for Git repositories.
//! While it is possible to provide your own definitions for
//! language detection, Gengo comes with a set of built-in
//! definitions.
//!
//! # Built-in Languages
#![doc = include_str!(concat!(env!("OUT_DIR"), "/language-list.md"))]

pub use analysis::Analysis;
pub use builder::Builder;
use documentation::Documentation;
pub use error::{Error, ErrorKind};
use generated::Generated;
use gix::attrs::StateRef;
use gix::bstr::{BString, ByteSlice};
use gix::prelude::FindExt;
use glob::MatchOptions;
pub use languages::analyzer::Analyzers;
use languages::Category;
pub use languages::Language;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::Ordering;
use vendored::Vendored;

pub mod analysis;
mod builder;
mod documentation;
mod error;
mod generated;
pub mod languages;
mod vendored;

type Result<T, E = Box<dyn std::error::Error + Send + Sync + 'static>> = std::result::Result<T, E>;

/// Shared match options for consistent behavior.
const GLOB_MATCH_OPTIONS: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

/// The main entry point for Gengo.
pub struct Gengo {
    repository: gix::ThreadSafeRepository,
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
}

impl GitState {
    fn new(repo: &gix::Repository, tree_id: &gix::oid) -> Result<(Self, gix::index::State)> {
        let index = repo.index_from_tree(tree_id)?;
        let attr_stack = repo.attributes_only(
            &index,
            gix::worktree::stack::state::attributes::Source::IdMapping,
        )?;
        let attr_matches = attr_stack.selected_attribute_matches([
            "gengo-language",
            "gengo-generated",
            "gengo-documentation",
            "gengo-vendored",
            "gengo-detectable",
        ]);
        Ok((
            Self {
                attr_stack: attr_stack.detach(),
                attr_matches,
            },
            index.into_parts().0,
        ))
    }
}

struct BlobEntry {
    // Just for path and id access
    index_entry: gix::index::Entry,
    result: Option<Entry>,
}

/// The result of analyzing a repository or a single submodule
struct Results {
    /// If this is a submodule, the root is not empty and the full path to where our paths start.
    root: BString,
    entries: Vec<BlobEntry>,
    path_storage: gix::index::PathStorage,
}

impl Results {
    /// Create a data structure that holds index entries as well as our results per entry.
    /// Return a list of paths at which submodules can be found, along with their
    /// commit ids.
    fn from_index(root: BString, index: gix::index::State) -> Self {
        use gix::index::entry::Mode;

        let (entries, path_storage) = index.into_entries();
        let entries: Vec<_> = entries
            .into_iter()
            .filter(|e| matches!(e.mode, Mode::FILE | Mode::FILE_EXECUTABLE))
            .map(|e| BlobEntry {
                index_entry: e,
                result: None,
            })
            .collect();

        Results {
            root,
            entries,
            path_storage,
        }
    }
}

impl Gengo {
    /// Analyzes each file in the repository at the given revision.
    pub fn analyze(&self, rev: &str) -> Result<Analysis> {
        let repo = self.repository.to_thread_local();
        let tree_id = repo.rev_parse_single(rev)?.object()?.peel_to_tree()?.id;
        let mut stack = vec![(BString::default(), repo, tree_id)];

        let mut all_results = Vec::new();
        // TODO Remove unused variables/code
        while let Some((root, repo, tree_id)) = stack.pop() {
            let is_submodule = false;
            let (state, index) = GitState::new(&repo, &tree_id)?;
            let mut results = Results::from_index(root.clone(), index);

            self.analyze_index(&repo.into_sync(), &mut results, state, is_submodule)?;
            all_results.push(results);
        }

        Ok(Analysis(all_results))
    }

    fn analyze_index(
        &self,
        repo: &gix::ThreadSafeRepository,
        results: &mut Results,
        state: GitState,
        is_submodule: bool,
    ) -> Result<()> {
        gix::parallel::in_parallel_with_slice(
            &mut results.entries,
            None,
            move |_| (state.clone(), repo.to_thread_local()),
            |entry, (state, repo), _, should_interrupt| {
                if should_interrupt.load(Ordering::Relaxed) {
                    return Ok(());
                }
                let Ok(path) =
                    gix::path::try_from_bstr(entry.index_entry.path_in(&results.path_storage))
                else {
                    return Ok(());
                };
                self.analyze_blob(path, repo, state, entry, is_submodule)
            },
            || Some(std::time::Duration::from_micros(5)),
            std::convert::identity,
        )?;
        Ok(())
    }

    fn analyze_blob(
        &self,
        filepath: impl AsRef<Path>,
        repo: &gix::Repository,
        state: &mut GitState,
        result: &mut BlobEntry,
        is_submodule: bool,
    ) -> Result<()> {
        let filepath = filepath.as_ref();
        let blob = repo.find_object(result.index_entry.id)?;
        let contents = blob.data.as_slice();
        state
            .attr_stack
            .at_path(filepath, Some(false), |id, buf| {
                repo.objects.find_blob(id, buf)
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

        // NOTE Unspecified attributes are None, so `state.is_set()` is
        //      implicitly `!state.is_unset()`.
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
            .unwrap_or_else(|| is_submodule || self.is_vendored(filepath, contents));

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
        result.result = Some(entry);
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
