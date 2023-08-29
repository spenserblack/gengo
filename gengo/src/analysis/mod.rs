use super::Entry;
use indexmap::map::Iter as IndexMapIter;
use indexmap::IndexMap;
use std::path::PathBuf;

pub use summary::Iter as SummaryIter;
pub use summary::Summary;

mod summary;

/// The result of analyzing a directory.
#[derive(Debug)]
pub struct Analysis(pub(super) IndexMap<PathBuf, Entry>);

impl Analysis {
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter())
    }

    /// Summarizes the analysis by language and size. Includes only
    /// the entries that are detectable.
    pub fn summary(&self) -> Summary {
        let mut summary = IndexMap::new();
        for entry in self.0.values().filter(|e| e.detectable()) {
            let language = entry.language().clone();
            *summary.entry(language).or_insert(0) += entry.size();
        }
        Summary(summary)
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
