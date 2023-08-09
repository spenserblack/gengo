use clap::Error as ClapError;
use clap::Parser;
use std::io::{Write, self};
use gengo::{Builder, languages::Category};
use std::collections::HashMap;

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
    #[arg(short='R', long, default_value = ".")]
    repository: String,
    /// The git revision to analyze.
    #[arg(short='r', long="rev", default_value = "HEAD")]
    revision: String,
    /// The maximum number of bytes to read from each file.
    ///
    /// This is useful for large files that can impact performance.
    ///
    /// The format is in bytes. The default is 1 MiB.
    #[arg(short='l', long, default_value = "1048576")]
    read_limit: usize,
}

impl CLI {
    pub fn run<Out: Write, Err: Write>(
        &self,
        mut out: Out,
        mut err: Err,
    ) -> Result<(), io::Error> {
        let gengo = Builder::new(&self.repository)
            .read_limit(self.read_limit)
            .build();
        let gengo = match gengo {
            Ok(gengo) => gengo,
            Err(e) => {
                writeln!(err, "failed to create instance: {}", e)?;
                return Ok(())
            }
        };
        let results = gengo.analyze(&self.revision);
        let results = match results {
            Ok(results) => results,
            Err(e) => {
                writeln!(err, "failed to analyze repository: {}", e)?;
                return Ok(())
            }
        };

        let mut compiled = HashMap::new();
        let mut total = 0;
        for (_, entry) in results.into_iter() {
            if entry.generated() || entry.vendored() || entry.documentation() {
                continue;
            }

            let language = entry.language();
            match language.category() {
                Category::Data | Category::Prose => continue,
                Category::Programming | Category::Markup => (),
            }

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
            writeln!(out, "{:.2}% {} {:>10}", percentage, size, language)?;
        }

        Ok(())
    }
}
