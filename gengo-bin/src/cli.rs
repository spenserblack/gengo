use clap::Error as ClapError;
use clap::Parser;
use gengo::{Analysis, Builder};
use indexmap::IndexMap;
use std::io::{self, Write};

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
    /// Force the output to not have colors.
    #[cfg(feature = "color")]
    #[arg(long)]
    no_color: bool,
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

            let language_str = language.name();
            let language_str = String::from(language_str);
            let size = entry.size();

            #[cfg(feature = "color")]
            let color = language.owo_color().unwrap();

            #[cfg(not(feature = "color"))]
            let color = ();

            let entry = compiled.entry(language_str).or_insert((0, color));
            entry.0 += size;
            total += size;
        }

        let total = total as f64;
        for (language, (size, color)) in compiled.into_iter() {
            let percentage = (size * 100) as f64 / total;
            let stats = format!("{:>6.2}% {}", percentage, size);
            let line = format!("{:<15} {}", stats, language);
            let line = self.colorize(&line, color);
            writeln!(out, "{}", line)?;
        }

        if self.breakdown {
            writeln!(out)?;
            self.run_breakdown(out, err, results)?;
        }

        Ok(())
    }

    fn run_breakdown<Out: Write, Err: Write>(
        &self,
        mut out: Out,
        mut _err: Err,
        results: Analysis,
    ) -> Result<(), io::Error> {
        let files_per_language = {
            let mut files_per_language = IndexMap::new();
            for (path, entry) in results.into_iter() {
                if !(self.all || entry.detectable()) {
                    continue;
                }
                let language = entry.language();

                #[cfg(feature = "color")]
                let color = language.owo_color().unwrap();

                #[cfg(not(feature = "color"))]
                let color = ();

                let language = language.name();

                let language = self.colorize(language, color);

                let language_files = files_per_language.entry(language).or_insert_with(Vec::new);
                let path_str = path.display().to_string();

                #[cfg(feature = "color")]
                let path_str = self.colorize(&path_str, color);

                language_files.push(path_str);
            }
            files_per_language
        };

        for (language, files) in files_per_language.into_iter() {
            writeln!(out, "{}", language)?;
            for file in files {
                writeln!(out, "  {}", file)?;
            }
            writeln!(out)?;
        }
        Ok(())
    }

    #[cfg(feature = "color")]
    fn colorize(&self, s: &str, color: owo_colors::Rgb) -> String {
        use owo_colors::{OwoColorize, Rgb};

        if self.no_color {
            return String::from(s);
        }

        // NOTE Adapted from https://css-tricks.com/converting-color-spaces-in-javascript/#aa-rgb-to-hsl
        let r = color.0;
        let g = color.1;
        let b = color.2;
        let min: u16 = [r, g, b].into_iter().min().unwrap().into();
        let max: u16 = [r, g, b].into_iter().max().unwrap().into();
        let lightness = (max + min) / 2;
        let bright = lightness > 0x7F;

        let line = s.on_color(color);
        let fg = if bright {
            Rgb(0, 0, 0)
        } else {
            Rgb(0xFF, 0xFF, 0xFF)
        };
        line.color(fg).to_string()
    }

    #[cfg(not(feature = "color"))]
    fn colorize<Anything>(&self, s: &str, _: Anything) -> String {
        String::from(s)
    }
}
