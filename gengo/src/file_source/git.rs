use super::{FileSource, Result, Overrides};
use gix::{
    attrs::StateRef,
    bstr::{BString, ByteSlice},
    Repository, ThreadSafeRepository,
    prelude::FindExt,
    discover::Error as DiscoverError,
    worktree::{
        stack::state::attributes::Source as AttrSource,
        Stack as WTStack
    },
    attrs::search::Outcome as AttrOutcome,
    index,
};
use crate::{Error, ErrorKind, GenericError};
use std::path::Path;
use std::borrow::Cow;
use std::error::Error as ErrorTrait;

struct Builder {
    repository: ThreadSafeRepository,
    rev: String,
}

impl Builder {
    fn new<P: AsRef<Path>>(path: P, rev: &str) -> Result<Self> {
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
    fn state(&self) -> Result<State> {
        let repo = self.repository.to_thread_local();
        let tree_id = repo.rev_parse_single(self.rev.as_str())?.object()?.peel_to_tree()?.id;
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

    fn build(self) -> Result<Git> {
        let state = self.state()?;
        let git = Git {
            repository: self.repository,
            rev: self.rev,
            state,
        };
        Ok(git)
    }
}

pub struct Git {
    repository: ThreadSafeRepository,
    rev: String,
    state: State,
}

impl Git {
    const OVERRIDE_ATTRS: [&'static str;5] = ["gengo-language", "gengo-documentation", "gengo-generated", "gengo-vendored", "gengo-detectable"];
    const LANGUAGE_OVERRIDE: usize = 0;
    const DOCUMENTATION_OVERRIDE: usize = 1;
    const GENERATED_OVERRIDE: usize = 2;
    const VENDORED_OVERRIDE: usize = 3;
    const DETECTABLE_OVERRIDE: usize = 4;
    pub fn new<P: AsRef<Path>>(path: P, rev: &str) -> Result<Self> {
        Builder::new(path, rev)?.build()
    }
}

impl<'repo> FileSource<'repo> for Git {
    type Filepath = Cow<'repo, Path>;
    type Contents = &'repo [u8];
    type Iter = Iter<'repo>;

    fn files(&self) -> Result<Self::Iter> {
        let path_storage = self.state.index_state.path_backing();
        todo!("Initialize iterator");
        let iter = Iter {
            stack: (),
            path_storage,
        };
        Ok(iter)
    }

    fn overrides<O: AsRef<Path>>(
        &self,
        path: O,
    ) -> Overrides {
        let repo = self.repository.to_thread_local();
        let state = {
            let mut state = self.state.clone();
            let Ok(platform) = state.attr_stack.at_path(path, Some(false), |id, buf| {
                repo.objects.find_blob(id, buf)
            }) else {
                // NOTE If we cannot get overrides, simply don't return them.
                return Default::default();
            };
            platform.matching_attributes(&mut state.attr_matches);
            state
        };

        let attrs = {
            let mut attrs = [None, None, None, None, None];
            state.attr_matches.iter_selected().zip(attrs.iter_mut()).for_each(|(info, slot)| {
                *slot = (info.assignment.state != StateRef::Unspecified).then_some(info);
            });
            attrs
        };

        let language = attrs[Self::LANGUAGE_OVERRIDE]
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
    stack: (),
    path_storage: &'repo index::PathStorage,
}

impl<'repo> Iterator for Iter<'repo> {
    type Item = (Cow<'repo, Path>, &'repo [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        todo!("implement iteration");
    }
}

#[derive(Clone)]
struct State {
    attr_stack: WTStack,
    attr_matches: AttrOutcome,
    index_state: index::State,
}
