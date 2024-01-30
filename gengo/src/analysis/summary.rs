use crate::LanguageOld;
use indexmap::map::Iter as IndexMapIter;
use indexmap::IndexMap;

/// The summary of an analysis.
#[derive(Debug)]
pub struct Summary(pub(super) IndexMap<LanguageOld, usize>);

impl Summary {
    /// Returns the total size of all languages.
    pub fn total(&self) -> usize {
        self.0.values().sum()
    }

    /// Returns an iterator over the languages and their sizes.
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter())
    }
}

pub struct Iter<'map>(IndexMapIter<'map, LanguageOld, usize>);

impl<'map> Iterator for Iter<'map> {
    type Item = (&'map LanguageOld, &'map usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'map> IntoIterator for &'map Summary {
    type Item = (&'map LanguageOld, &'map usize);
    type IntoIter = Iter<'map>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Options to use when creating a summary.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Opts {
    /// Include all files, even if they are not detectable.
    pub all: bool,
}
