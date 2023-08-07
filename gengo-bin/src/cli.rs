use clap::Parser;
use std::error::Error;
use std::io::Write;

pub fn new() -> Cli {
    Cli::parse()
}

pub fn new_from(args: &[&str]) -> Cli {
    Cli::parse_from(args)
}

/// Fetch language statistics for your source code.
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// The path to the repository to analyze.
    #[arg(short, long, default_value = ".")]
    repository: String,
}

impl Cli {
    pub fn run<W: Write>(&self, mut w: W) -> Result<(), Box<dyn Error>> {
        // TODO Implement `Display` and read the `to_string` value?
        writeln!(w, "Would read from {}", self.repository,)?;
        Ok(())
    }
}
