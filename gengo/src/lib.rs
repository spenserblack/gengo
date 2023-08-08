use git2::Repository;
use std::path::Path;
use std::error::Error;
pub use languages::analyzer::Analyzers;
pub use languages::Language;
pub use builder::Builder;

pub mod languages;
mod builder;

/// The main entry point for Gengo.
pub struct Gengo {
    repository: Repository,
    analyzers: Analyzers,
}
