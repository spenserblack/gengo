use super::{FileSource, Overrides};
use crate::{Error, ErrorKind};
use gix::{
    attrs::search::Outcome as AttrOutcome,
    attrs::StateRef,
    bstr::ByteSlice,
    discover::Error as DiscoverError,
    index,
    prelude::FindExt,
    worktree::{stack::state::attributes::Source as AttrSource, Stack as WTStack},
    ThreadSafeRepository,
};
use std::borrow::Cow;
use std::path::Path;

use std::slice;

struct Builder {
    repository: ThreadSafeRepository,
    rev: String,
}

impl Builder {
    fn new<P: AsRef<Path>>(path: P, rev: &str) -> crate::Result<Self> {
        let repository = match gix::discover(path) {
            Ok(r) => r,
            Err(DiscoverError::Discover(err)) => {
                return Err(Box::new(Error::with_source(ErrorKind::NoRepository, err)))
            }
            Err(err) => return Err(err.into()),
        };

        let repository = repository.into_sync();
        let rev = rev.to_string();
        Ok(Self { repository, rev })
    }

    /// Constructs a [`State`] for the repository and rev.
    fn state(&self) -> crate::Result<State> {
        let repo = self.repository.to_thread_local();
        let tree_id = repo
            .rev_parse_single(self.rev.as_str())?
            .object()?
            .peel_to_tree()?
            .id;
        let index = repo.index_from_tree(&tree_id)?;
        let attr_stack = repo.attributes_only(&index, AttrSource::IdMapping)?;
        let attr_matches = attr_stack.selected_attribute_matches(Git::OVERRIDE_ATTRS);
        let (index_state, _) = index.into_parts();
        let attr_stack = attr_stack.detach();
        let state = State {
            attr_stack,
            attr_matches,
            index_state,
        };
        Ok(state)
    }

    fn build(self) -> crate::Result<Git> {
        let state = self.state()?;
        let git = Git {
            repository: self.repository,
            state,
        };
        Ok(git)
    }
}

pub struct Git {
    repository: ThreadSafeRepository,
    state: State,
}

impl Git {
    const OVERRIDE_ATTRS: [&'static str; 5] = [
        "gengo-language",
        "gengo-documentation",
        "gengo-generated",
        "gengo-vendored",
        "gengo-detectable",
    ];
    const LANGUAGE_OVERRIDE: usize = 0;
    const DOCUMENTATION_OVERRIDE: usize = 1;
    const GENERATED_OVERRIDE: usize = 2;
    const VENDORED_OVERRIDE: usize = 3;
    const DETECTABLE_OVERRIDE: usize = 4;
    pub fn new<P: AsRef<Path>>(path: P, rev: &str) -> crate::Result<Self> {
        Builder::new(path, rev)?.build()
    }
}

impl<'repo> FileSource<'repo> for Git {
    type Filepath = Cow<'repo, Path>;
    type Contents = Vec<u8>;
    type FileEntry = &'repo index::Entry;
    type Iter = Iter<'repo>;

    fn files(&'repo self) -> crate::Result<Self::Iter> {
        let entries = self.state.index_state.entries().iter();
        let iter = Iter { entries };
        Ok(iter)
    }

    fn filename(&'repo self, entry: &Self::FileEntry) -> crate::Result<Self::Filepath> {
        let path_storage = self.state.index_state.path_backing();
        let path = entry.path_in(path_storage);
        let path = gix::path::try_from_bstr(path)?;
        Ok(path)
    }

    fn contents(&'repo self, entry: &Self::FileEntry) -> crate::Result<Self::Contents> {
        let repository = self.repository.to_thread_local();
        let blob = repository.find_object(entry.id)?;
        let contents = blob.detach().data;
        Ok(contents)
    }

    fn overrides<O: AsRef<Path>>(&self, path: O) -> Overrides {
        let repo = self.repository.to_thread_local();
        let state = {
            let mut state = self.state.clone();
            let Ok(platform) = state
                .attr_stack
                .at_path(path, Some(false), |id, buf| repo.objects.find_blob(id, buf))
            else {
                // NOTE If we cannot get overrides, simply don't return them.
                return Default::default();
            };
            platform.matching_attributes(&mut state.attr_matches);
            state
        };

        let attrs = {
            let mut attrs = [None, None, None, None, None];
            state
                .attr_matches
                .iter_selected()
                .zip(attrs.iter_mut())
                .for_each(|(info, slot)| {
                    *slot = (info.assignment.state != StateRef::Unspecified).then_some(info);
                });
            attrs
        };

        let language =
            attrs[Self::LANGUAGE_OVERRIDE]
                .as_ref()
                .and_then(|info| match info.assignment.state {
                    StateRef::Value(v) => v.as_bstr().to_str().ok().map(|s| s.replace('-', " ")),
                    _ => None,
                });
        // NOTE Unspecified attributes are None, so `state.is_set()` is
        //      implicitly `!state.is_unset()`.
        // TODO This is really repetitive. Refactor to iteration?
        let is_documentation = attrs[Self::DOCUMENTATION_OVERRIDE]
            .as_ref()
            .map(|info| info.assignment.state.is_set());
        let is_generated = attrs[Self::GENERATED_OVERRIDE]
            .as_ref()
            .map(|info| info.assignment.state.is_set());
        let is_vendored = attrs[Self::VENDORED_OVERRIDE]
            .as_ref()
            .map(|info| info.assignment.state.is_set());
        let is_detectable = attrs[Self::DETECTABLE_OVERRIDE]
            .as_ref()
            .map(|info| info.assignment.state.is_set());

        Overrides {
            language,
            is_documentation,
            is_generated,
            is_vendored,
            is_detectable,
        }
    }
}

pub struct Iter<'repo> {
    entries: slice::Iter<'repo, index::Entry>,
}

impl<'repo> Iterator for Iter<'repo> {
    type Item = &'repo index::Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }
}

#[derive(Clone)]
struct State {
    attr_stack: WTStack,
    attr_matches: AttrOutcome,
    index_state: index::State,
}
