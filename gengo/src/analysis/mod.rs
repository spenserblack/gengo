use super::Entry;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::fmt::Formatter;
use std::path::Path;

pub use summary::Iter as SummaryIter;
pub use summary::Opts as SummaryOpts;
pub use summary::Summary;

mod summary;

/// The result of analyzing a directory.
pub struct Analysis(pub(super) crate::Results);

impl Analysis {
    pub fn iter(&self) -> impl Iterator<Item = (Cow<'_, Path>, &Entry)> + '_ {
        self.0.entries.iter().filter_map(|entry| {
            entry.result.as_ref().map(|result| {
                (
                    gix::path::from_bstr(entry.index_entry.path_in(&self.0.path_storage)),
                    result,
                )
            })
        })
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
        for entry in self.0.entries.iter().filter_map(|e| e.result.as_ref()) {
            if !(opts.all || entry.detectable()) {
                continue;
            }
            let language = entry.language().clone();
            *summary.entry(language).or_insert(0) += entry.size();
        }
        Summary(summary)
    }
}

impl std::fmt::Debug for Analysis {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let map = f
            .debug_map()
            .entries(self.0.entries.iter().filter_map(|e| {
                e.result
                    .as_ref()
                    .map(|result| (e.index_entry.path_in(&self.0.path_storage), result))
            }))
            .finish()?;
        f.debug_tuple("Analysis").field(&map).finish()
    }
}
