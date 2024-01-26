use super::Entry;
use indexmap::IndexMap;
use serde::Serialize;

use std::path::PathBuf;

pub use summary::Iter as SummaryIter;
pub use summary::Opts as SummaryOpts;
pub use summary::Summary;

mod summary;

/// The result of analyzing a repository along with all of its submodules.
#[derive(Debug, Serialize)]
pub struct Analysis(pub(super) IndexMap<PathBuf, Entry>);

impl Analysis {
    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &Entry)> {
        let results = &self.0;
        results.iter()
    }

    /// Summarizes the analysis by language and size. Includes only
    /// the entries that are detectable.
    pub fn summary(&self) -> Summary {
        let opts = SummaryOpts {
            all: false,
            ..Default::default()
        };
        self.summary_with(opts)
    }

    /// Summarizes the analysis by language and size.
    pub fn summary_with(&self, opts: SummaryOpts) -> Summary {
        let mut summary = IndexMap::new();
        for (_, entry) in self.iter() {
            if !(opts.all || entry.detectable()) {
                continue;
            }
            let language = entry.language().clone();
            *summary.entry(language).or_insert(0) += entry.size();
        }
        Summary(summary)
    }
}
