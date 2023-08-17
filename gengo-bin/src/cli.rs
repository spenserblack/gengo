use clap::Error as ClapError;
use clap::Parser;
use gengo::{Builder, Entry};
use indexmap::IndexMap;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn new() -> CLI {
    CLI::parse()
}

pub fn try_new_from(args: &[&str]) -> Result<CLI, ClapError> {
    CLI::try_parse_from(args)
}

/// Fetch language statistics for your source code.
#[derive(Parser)]
#[command(version)]
pub struct CLI {
    /// The path to the repository to analyze.
    #[arg(short = 'R', long, default_value = ".")]
    repository: String,
    /// The git revision to analyze.
    #[arg(short = 'r', long = "rev", default_value = "HEAD")]
    revision: String,
    /// The maximum number of bytes to read from each file.
    ///
    /// This is useful for large files that can impact performance.
    ///
    /// The format is in bytes. The default is 1 MiB.
    #[arg(short = 'l', long, default_value = "1048576")]
    read_limit: usize,
    /// Report on all files, even if they are not detectable.
    #[arg(short = 'a', long)]
    all: bool,
    /// Include detailed statistics for each language.
    #[arg(short = 'b', long)]
    breakdown: bool,
}

impl CLI {
    pub fn run<Out: Write, Err: Write>(&self, mut out: Out, mut err: Err) -> Result<(), io::Error> {
        let gengo = Builder::new(&self.repository)
            .read_limit(self.read_limit)
            .build();
        let gengo = match gengo {
            Ok(gengo) => gengo,
            Err(e) => {
                writeln!(err, "failed to create instance: {}", e)?;
                return Ok(());
            }
        };
        let results = gengo.analyze(&self.revision);
        let results = match results {
            Ok(results) => results,
            Err(e) => {
                writeln!(err, "failed to analyze repository: {}", e)?;
                return Ok(());
            }
        };

        let mut compiled = IndexMap::new();
        let mut total = 0;
        for (_, entry) in results.iter() {
            if !(self.all || entry.detectable()) {
                continue;
            }

            let language = entry.language();

            let language = language.name();
            let language = String::from(language);
            let size = entry.size();

            let entry = compiled.entry(language).or_insert(0);
            *entry += size;
            total += size;
        }

        let total = total as f64;
        for (language, size) in compiled.into_iter() {
            let percentage = (size * 100) as f64 / total;
            let stats = format!("{:>6.2}% {}", percentage, size);
            writeln!(out, "{:<15} {}", stats, language)?;
        }

        if self.breakdown {
            writeln!(out)?;
            Self::run_breakdown(out, err, results)?;
        }

        Ok(())
    }

    fn run_breakdown<Out: Write, Err: Write>(
        mut out: Out,
        mut _err: Err,
        results: IndexMap<PathBuf, Entry>,
    ) -> Result<(), io::Error> {
        let files_per_language = {
            let mut files_per_language = IndexMap::new();
            for (path, entry) in results.into_iter() {
                let language = entry.language();
                let language = language.name();
                let language = String::from(language);

                let language_files = files_per_language.entry(language).or_insert_with(Vec::new);
                language_files.push(path);
            }
            files_per_language
        };

        for (language, files) in files_per_language.into_iter() {
            writeln!(out, "{}", language)?;
            for file in files {
                writeln!(out, "  {}", file.display())?;
            }
            writeln!(out)?;
        }
        Ok(())
    }
}
