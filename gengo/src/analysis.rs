use super::Entry;
use indexmap::map::Iter as IndexMapIter;
use indexmap::IndexMap;
use std::path::PathBuf;

/// The result of analyzing a directory.
#[derive(Debug)]
pub struct Analysis(pub(super) IndexMap<PathBuf, Entry>);

impl Analysis {
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter())
    }
}

pub struct Iter<'map>(IndexMapIter<'map, PathBuf, Entry>);

impl<'map> Iterator for Iter<'map> {
    type Item = (&'map PathBuf, &'map Entry);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'map> IntoIterator for &'map Analysis {
    type Item = (&'map PathBuf, &'map Entry);
    type IntoIter = Iter<'map>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
