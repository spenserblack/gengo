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
    /// The left side of the equation.
    left: usize,
    /// The right side of the equation.
    right: usize,
}

impl Cli {
    pub fn run<W: Write>(&self, mut w: W) -> Result<(), Box<dyn Error>> {
        // TODO Implement `Display` and read the `to_string` value?
        let result = gengo::add(self.left, self.right);
        writeln!(
            w,
            "{left} + {right} = {result}",
            left = self.left,
            right = self.right
        )?;
        Ok(())
    }
}
