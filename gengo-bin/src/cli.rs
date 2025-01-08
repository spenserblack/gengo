use clap::Error as ClapError;
use clap::{Parser, Subcommand, ValueEnum};
use gengo::{analysis::SummaryOpts, Analysis, Builder, Directory, Git};
use indexmap::IndexMap;
use std::error::Error as BaseError;
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
#[command(propagate_version = true)]
pub struct CLI {
    #[command(subcommand)]
    command: Commands,
    /// The maximum number of bytes to read from each file.
    ///
    /// This is useful for large files that can impact performance.
    ///
    /// The format is in bytes. The default is 1 MiB.
    #[arg(short = 'l', long, default_value = "1048576")]
    read_limit: usize,
    /// Report on all files, even if they are not detectable.
    ///
    /// This only applies to the pretty format, as machine-readable
    /// formats always include all files.
    #[arg(short = 'a', long)]
    all: bool,
    /// Include detailed statistics for each language.
    ///
    /// This only applies to the pretty format, as machine-readable
    /// formats always include detailed statistics.
    #[arg(short = 'b', long)]
    breakdown: bool,
    /// Control when colors are displayed.
    #[cfg(feature = "color")]
    #[arg(long, default_value = "always")]
    color: ColorControl,
    /// The format to use for output.
    #[arg(short = 'F', long, default_value = "pretty")]
    format: Format,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a git repository.
    Git {
        /// The path to the repository to analyze.
        #[arg(short = 'R', long, default_value = ".")]
        repository: String,
        /// The git revision to analyze.
        #[arg(short = 'r', long = "rev", default_value = "HEAD")]
        revision: String,
    },
    /// ***BETA*** Analyze a directory.
    Directory {
        /// The path to the directory to analyze.
        #[arg(short = 'D', long, default_value = ".")]
        directory: String,
    },
}

#[cfg(feature = "color")]
#[derive(ValueEnum, Debug, Clone)]
enum ColorControl {
    /// Always use colors.
    Always,
    /// Use only the 8 ANSI colors.
    Ansi,
    /// Disable colors.
    Never,
}

#[derive(ValueEnum, Debug, Clone)]
enum Format {
    /// Output for humans.
    Pretty,
    /// JSON output.
    Json,
}

impl CLI {
    pub fn run(&self, mut out: impl Write, mut err: impl Write) -> Result<(), io::Error> {
        let results = self.command.analyze(self.read_limit);
        let results = match results {
            Ok(results) => results,
            Err(e) => {
                writeln!(err, "failed to analyze repository: {}", e)?;
                return Ok(());
            }
        };

        match self.format {
            Format::Pretty => (),
            Format::Json => return self.run_json(results, out, err),
        }

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
            let color = language.owo_color();
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

    fn run_json(
        &self,
        analysis: Analysis,
        mut out: impl Write,
        mut _err: impl Write,
    ) -> Result<(), io::Error> {
        match serde_json::to_string(&analysis) {
            Ok(s) => writeln!(out, "{s}")?,
            Err(e) => {
                writeln!(out, "failed to serialize to JSON: {e}")?;
                return Ok(());
            }
        };
        Ok(())
    }

    fn run_breakdown(
        &self,
        mut out: impl Write,
        mut _err: impl Write,
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
            let color = language.owo_color();
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
        use owo_colors::{AnsiColors::*, OwoColorize, Rgb};
        use ColorControl::*;

        match self.color {
            Never => String::from(s),
            Always => {
                let fg = Self::is_bright(color.0, color.1, color.2)
                    .then_some(Rgb(0, 0, 0))
                    .unwrap_or(Rgb(0xFF, 0xFF, 0xFF));
                s.on_color(color).color(fg).to_string()
            }
            Ansi => {
                let (bg, (r, g, b)) = Self::closest_color(color);
                let fg = Self::is_bright(r, g, b).then_some(Black).unwrap_or(White);
                s.on_color(bg).color(fg).to_string()
            }
        }
    }

    #[cfg(feature = "color")]
    fn is_bright(r: u8, g: u8, b: u8) -> bool {
        // NOTE Adapted from https://css-tricks.com/converting-color-spaces-in-javascript/#aa-rgb-to-hsl
        let min: u16 = [r, g, b].into_iter().min().unwrap().into();
        let max: u16 = [r, g, b].into_iter().max().unwrap().into();
        let lightness = (max + min) / 2;
        lightness > 0x7F
    }

    #[cfg(feature = "color")]
    fn closest_color(rgb: owo_colors::Rgb) -> (owo_colors::AnsiColors, (u8, u8, u8)) {
        use owo_colors::AnsiColors::*;
        // NOTE Gets the closest color by Euclidean distance
        [
            (Black, (0u8, 0u8, 0u8)),
            (Red, (0xFF, 0, 0)),
            (Yellow, (0xFF, 0xFF, 0)),
            (Blue, (0, 0, 0xFF)),
            (Magenta, (0xFF, 0, 0xFF)),
            (Cyan, (0, 0xFF, 0xFF)),
            (White, (0xFF, 0xFF, 0xFF)),
        ]
        .into_iter()
        .min_by_key(|(_, (r, g, b))| {
            // NOTE As a shortcut we'll just skip the square root step
            [(r, rgb.0), (g, rgb.1), (b, rgb.2)]
                .into_iter()
                .map(|(p1, p2)| u32::from(p1.abs_diff(p2)))
                .map(|diff| diff * diff)
                .sum::<u32>()
        })
        .unwrap()
    }

    #[cfg(not(feature = "color"))]
    fn colorize<Anything>(&self, s: &str, _: Anything) -> String {
        String::from(s)
    }
}

impl Commands {
    fn analyze(&self, read_limit: usize) -> Result<Analysis, Box<dyn BaseError>> {
        match self {
            Commands::Git {
                repository,
                revision,
            } => {
                let git = Git::new(repository, revision)?;
                let gengo = Builder::new(git).read_limit(read_limit).build()?;
                gengo.analyze()
            }
            Commands::Directory { directory } => {
                let directory = Directory::new(directory, read_limit)?;
                let gengo = Builder::new(directory).read_limit(read_limit).build()?;
                gengo.analyze()
            }
        }
    }
}
