use super::{FileSource, Result};
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
    index::State as IndexState,
};
use crate::{Error, ErrorKind, GenericError};
use std::path::Path;
use std::borrow::Cow;
use std::error::Error as ErrorTrait;

pub struct Git {
    repository: ThreadSafeRepository,
    rev: String,
}

impl Git {
    const OVERRIDE_ATTRS: [&'static str;5] = ["gengo-language", "gengo-generated", "gengo-documentation", "gengo-vendored", "gengo-detectable"];
    const LANGUAGE_OVERRIDE: usize = 0;
    const GENERATED_OVERRIDE: usize = 1;
    const DOCUMENTATION_OVERRIDE: usize = 2;
    const VENDORED_OVERRIDE: usize = 3;
    const DETECTABLE_OVERRIDE: usize = 4;
    pub fn new<P: AsRef<Path>>(path: P, rev: &str) -> Result<Self> {
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
        let attr_matches = attr_stack.selected_attribute_matches(Self::OVERRIDE_ATTRS);
        let (index_state, _) = index.into_parts();
        let attr_stack = attr_stack.detach();
        let state = State {
            attr_stack,
            attr_matches,
            index_state,
        };
        Ok(state)
    }
}

impl<'repo> FileSource<'repo> for Git {
    type Filepath = Cow<'repo, Path>;
    type Contents = &'repo [u8];
    type Iter = Iter<'repo>;

    fn files(&self) -> Result<Self::Iter> {
        todo!("Initialize iterator");
        let state = self.state()?;
        let iter = Iter {
            state,
            stack: (),
            foo: & (),
        };
        Ok(iter)
    }
}

pub struct Iter<'repo> {
    state: State,
    stack: (),
    foo: &'repo (),
}

impl<'repo> Iterator for Iter<'repo> {
    type Item = (Cow<'repo, Path>, &'repo [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        todo!("implement iteration");
    }
}

struct State {
    attr_stack: WTStack,
    attr_matches: AttrOutcome,
    index_state: IndexState,
}
