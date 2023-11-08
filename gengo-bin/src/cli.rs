use clap::Error as ClapError;
use clap::Parser;
use gengo::{analysis::SummaryOpts, Analysis, Builder, Directory, Git};
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
    /// Analyze a directory instead of a repository (BETA).
    #[arg(short = 'd', long, conflicts_with("revision"))]
    directory: bool,
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
        let results = if self.directory {
            let directory = match Directory::new(&self.repository, self.read_limit) {
                Ok(directory) => directory,
                Err(e) => {
                    writeln!(err, "failed to create directory instance: {}", e)?;
                    return Ok(());
                }
            };
            let gengo = Builder::new(directory).read_limit(self.read_limit).build();
            let gengo = match gengo {
                Ok(gengo) => gengo,
                Err(e) => {
                    writeln!(err, "failed to create instance: {}", e)?;
                    return Ok(());
                }
            };
            gengo.analyze()
        } else {
            let git = match Git::new(&self.repository, &self.revision) {
                Ok(git) => git,
                Err(e) => {
                    writeln!(err, "failed to create git instance: {}", e)?;
                    return Ok(());
                }
            };
            let gengo = Builder::new(git).read_limit(self.read_limit).build();
            let gengo = match gengo {
                Ok(gengo) => gengo,
                Err(e) => {
                    writeln!(err, "failed to create instance: {}", e)?;
                    return Ok(());
                }
            };
            gengo.analyze()
        };
        let results = match results {
            Ok(results) => results,
            Err(e) => {
                writeln!(err, "failed to analyze repository: {}", e)?;
                return Ok(());
            }
        };

        let mut summary_opts: SummaryOpts = Default::default();
        summary_opts.all = self.all;
        let summary = results.summary_with(summary_opts);
        let total = summary.total();
        let total = total as f64;

        let summary = {
            let mut summary: Vec<(_, _)> = summary.iter().collect();
            summary.sort_by_key(|(language, size)| (usize::MAX - *size, language.name()));
            summary
        };

        for (language, size) in summary.iter() {
            let percentage = (*size * 100) as f64 / total;
            #[cfg(feature = "color")]
            let color = language.owo_color().unwrap();
            #[cfg(not(feature = "color"))]
            let color = ();

            let stats = format!("{:>6.2}% {}", percentage, size);
            let line = format!("{:<15} {}", stats, language.name());
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
            for (path, entry) in results.iter() {
                if !(self.all || entry.detectable()) {
                    continue;
                }

                let language = entry.language();
                let language_files = files_per_language.entry(language).or_insert_with(Vec::new);
                language_files.push(path);
            }
            files_per_language
        };

        let files_per_language = {
            let mut v: Vec<(_, _)> = files_per_language.into_iter().collect();
            v.sort_by_key(|(language, _)| language.name());
            v
        };

        for (language, files) in files_per_language.into_iter() {
            #[cfg(feature = "color")]
            let color = language.owo_color().unwrap();
            #[cfg(not(feature = "color"))]
            let color = ();

            writeln!(out, "{}", self.colorize(language.name(), color))?;

            let files = {
                let mut files = files;
                files.sort();
                files
            };

            for file in files {
                writeln!(
                    out,
                    "  {}",
                    self.colorize(&file.display().to_string(), color)
                )?;
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
