use clap::Error as ClapError;
use clap::Parser;
use std::error::Error;
use std::io::Write;

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
    #[arg(short, long, default_value = ".")]
    repository: String,
}

impl CLI {
    pub fn run<Out: Write, Err: Write>(
        &self,
        mut out: Out,
        mut err: Err,
    ) -> Result<(), Box<dyn Error>> {
        writeln!(out, "Would read from {}", self.repository,)?;
        Ok(())
    }
}
