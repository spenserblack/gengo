use super::Entry;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::path::Path;

pub use summary::Iter as SummaryIter;
pub use summary::Opts as SummaryOpts;
pub use summary::Summary;

mod summary;

/// The result of analyzing a repository along with all of its submodules.
pub struct Analysis(pub(super) Vec<crate::Results>);

impl Analysis {
    pub fn iter(&self) -> impl Iterator<Item = (Cow<'_, Path>, &Entry)> + '_ {
        self.0.iter().flat_map(|results| {
            results.entries.iter().filter_map(|entry| {
                entry.result.as_ref().and_then(|result| {
                    Some((
                        {
                            let p = entry.index_entry.path_in(&results.path_storage);
                            if !results.root.is_empty() {
                                let mut base = results.root.clone();
                                base.push(b'/');
                                base.extend_from_slice(p);
                                gix::path::try_from_bstring(base).ok()?.into()
                            } else {
                                gix::path::try_from_bstr(p).ok()?
                            }
                        },
                        result,
                    ))
                })
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
        for entry in self
            .0
            .iter()
            .flat_map(|results| results.entries.iter().filter_map(|e| e.result.as_ref()))
        {
            if !(opts.all || entry.detectable()) {
                continue;
            }
            let language = entry.language().clone();
            *summary.entry(language).or_insert(0) += entry.size();
        }
        Summary(summary)
    }
}

impl Debug for Analysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Analysis ")?;
        f.debug_map().entries(self.iter()).finish()
    }
}
