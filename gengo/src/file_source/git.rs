use super::FileSource;
use std::path::Path;
use std::error::Error;
use git2::{AttrCheckFlags, AttrValue, Blob, Commit, ObjectType, Repository, Tree};

pub struct Git {
    repository: Repository,
}

impl<P: AsRef<Path>> FileSource<'bytes, P> for Git {
    type Error = Box<dyn Error>;
    type Iter = !; // TODO

    fn open<Dir: AsRef<Path>>(path: Dir) -> Result<Self, Self::Error> {
        let repository = Repository::open(path)?;
        let git = Git { repository };
        Ok(git)
    }

    fn iter(&self) -> Result<Self::Iter, Self::Error> {
        todo!("Open revision and iterate over files")
    }
}

pub struct Iter<'bytes> {
    tree: Tree<'bytes>,
}
