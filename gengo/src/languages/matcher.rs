use glob::Pattern;
use indexmap::IndexSet;
use once_cell::sync::Lazy;
use regex::Regex;

use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::path::Path;

/// Checks if a file matches.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Matcher {
    Filepath(Filepath),
    Shebang(Shebang),
}

/// Matches a file path.
#[derive(Clone, Debug)]
pub struct Filepath {
    extensions: IndexSet<OsString>,
    filenames: IndexSet<OsString>,
    patterns: Vec<Pattern>,
}

impl Filepath {
    /// Create a new filepath matcher.
    pub fn new<S: AsRef<OsStr>>(extensions: &[S], filenames: &[S], patterns: &[String]) -> Self {
        let extensions = extensions.iter().map(Into::into).collect();
        let filenames = filenames.iter().map(Into::into).collect();
        let patterns = patterns
            .iter()
            .map(|s| Pattern::new(s.as_ref()).unwrap())
            .collect();
        Self {
            extensions,
            filenames,
            patterns,
        }
    }

    pub fn matches_extension<P: AsRef<Path>>(&self, filename: P) -> bool {
        let extension = filename.as_ref().extension().unwrap_or_default();
        self.extensions.contains(extension)
    }

    pub fn matches_filename<P: AsRef<Path>>(&self, filename: P) -> bool {
        self.filenames
            .contains(filename.as_ref().file_name().unwrap_or_default())
    }

    pub fn matches_pattern<P: AsRef<Path>>(&self, filename: P) -> bool {
        self.patterns
            .iter()
            .any(|p| p.matches_path(filename.as_ref()))
    }

    pub fn matches<P: AsRef<Path>>(&self, filename: P) -> bool {
        self.matches_extension(&filename)
            || self.matches_filename(&filename)
            || self.matches_pattern(&filename)
    }
}

/// Matches a shebang.
#[derive(Clone, Debug)]
pub struct Shebang {
    interpreters: IndexSet<String>,
}

impl Shebang {
    const MAX_SHEBANG_LENGTH: usize = 50;

    pub fn new<S: Display>(interpreters: &[S]) -> Self {
        let interpreters = interpreters.iter().map(|s| s.to_string()).collect();
        Self { interpreters }
    }

    /// Checks if the file contents match a shebang by checking the first line of the contents.
    ///
    /// Does not read more than 100 bytes.
    pub fn matches(&self, contents: &[u8]) -> bool {
        let mut lines = contents.split(|&c| c == b'\n');
        let first_line = lines.next().unwrap_or_default();
        // Check that the first line is a shebang
        if first_line.len() < 2 || first_line[0] != b'#' || first_line[1] != b'!' {
            return false;
        }
        let first_line = if first_line.len() > Self::MAX_SHEBANG_LENGTH {
            &first_line[..Self::MAX_SHEBANG_LENGTH]
        } else {
            first_line
        };
        let first_line = String::from_utf8_lossy(first_line);
        // NOTE Handle trailing spaces, `\r`, etc.
        let first_line = first_line.trim_end();
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^#!(?:/usr(?:/local)?)?/bin/(?:env )?([\w\d]+)\r?$").unwrap()
        });

        RE.captures(first_line)
            .and_then(|c| c.get(1))
            .map_or(false, |m| {
                let interpreter = m.as_str();
                self.interpreters.contains(interpreter)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_matches_extension() {
        let analyzer = Filepath::new(&["txt"], &[], &[]);
        assert!(analyzer.matches("foo.txt"));
        assert!(!analyzer.matches("foo.rs"));
    }

    #[test]
    fn test_matches_filename() {
        let analyzer = Filepath::new(&[], &["LICENSE"], &[]);
        assert!(analyzer.matches("LICENSE"));
        assert!(!analyzer.matches("Dockerfile"));
    }

    #[rstest(
        pattern,
        filename,
        case("Makefile.*", "Makefile.in"),
        case(".vscode/*.json", ".vscode/extensions.json")
    )]
    fn test_matches_pattern(pattern: &str, filename: &str) {
        let analyzer = Filepath::new::<&str>(&[], &[], &[pattern.into()]);
        assert!(analyzer.matches(filename));
    }

    #[test]
    fn test_matches_shebang() {
        let analyzer = Shebang::new(&["python", "python3"]);
        assert!(analyzer.matches(b"#!/bin/python\n"));
        assert!(analyzer.matches(b"#!/usr/bin/python\n"));
        assert!(analyzer.matches(b"#!/usr/local/bin/python\n"));
        assert!(analyzer.matches(b"#!/usr/bin/python3\n"));
        assert!(analyzer.matches(b"#!/usr/bin/env python\n"));
        assert!(!analyzer.matches(b"#!/bin/sh\n"));
    }
}
