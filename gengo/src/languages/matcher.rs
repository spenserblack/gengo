use glob::Pattern;
use indexmap::IndexSet;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::GLOB_MATCH_OPTIONS;
use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::path::Path;

/// Checks if a file matches.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Matcher {
    Extension(Extension),
    Filename(Filename),
    FilepathPattern(FilepathPattern),
    Shebang(Shebang),
}

/// Matches a file extension.
#[derive(Clone, Debug)]
pub struct Extension {
    extensions: IndexSet<OsString>,
}

impl Extension {
    /// Create a new filepath matcher.
    pub fn new(extensions: &[impl AsRef<OsStr>]) -> Self {
        let extensions = extensions.iter().map(Into::into).collect();
        Self { extensions }
    }

    pub fn matches(&self, filename: impl AsRef<Path>) -> bool {
        self.extensions
            .contains(filename.as_ref().extension().unwrap_or_default())
    }
}

/// Matches a filename.
#[derive(Clone, Debug)]
pub struct Filename {
    filenames: IndexSet<OsString>,
}

impl Filename {
    /// Create a new filepath matcher.
    pub fn new(filenames: &[impl AsRef<OsStr>]) -> Self {
        let filenames = filenames.iter().map(Into::into).collect();
        Self { filenames }
    }

    pub fn matches(&self, filename: impl AsRef<Path>) -> bool {
        self.filenames
            .contains(filename.as_ref().file_name().unwrap_or_default())
    }
}

/// Matches a filepath pattern
#[derive(Clone, Debug)]
pub struct FilepathPattern {
    patterns: Vec<Pattern>,
}

impl FilepathPattern {
    /// Create a new filepath matcher.
    pub fn new(patterns: &[String]) -> Self {
        let patterns = patterns
            .iter()
            .map(|s| Pattern::new(s.as_ref()).unwrap())
            .collect();
        Self { patterns }
    }

    pub fn matches(&self, filename: impl AsRef<Path>) -> bool {
        self.patterns
            .iter()
            .any(|p| p.matches_path_with(filename.as_ref(), GLOB_MATCH_OPTIONS))
    }
}

/// Matches a shebang.
#[derive(Clone, Debug)]
pub struct Shebang {
    interpreters: IndexSet<String>,
}

impl Shebang {
    const MAX_SHEBANG_LENGTH: usize = 50;

    pub fn new(interpreters: &[impl Display]) -> Self {
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
            Regex::new(r"^#!(?:/usr(?:/local)?)?/bin/(?:env\s+)?([\w\d]+)\r?$").unwrap()
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
        let analyzer = Extension::new(&["txt"]);
        assert!(analyzer.matches("foo.txt"));
        assert!(!analyzer.matches("foo.rs"));
    }

    #[test]
    fn test_matches_filename() {
        let analyzer = Filename::new(&["LICENSE"]);
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
        let analyzer = FilepathPattern::new(&[pattern.into()]);
        assert!(analyzer.matches(filename));
    }

    #[rstest(pattern, filename, case("Makefile.*", "Makefile.in/foo"))]
    fn test_rejects_pattern(pattern: &str, filename: &str) {
        let analyzer = FilepathPattern::new(&[pattern.into()]);
        assert!(!analyzer.matches(filename));
    }

    #[rstest(
        pattern,
        matches,
        case(b"#!/bin/python\n", true),
        case(b"#!/usr/bin/python\n", true),
        case(b"#!/usr/local/bin/python\n", true),
        case(b"#!/usr/bin/python3\n", true),
        case(b"#!/usr/bin/env python\n", true),
        case(b"#!/usr/bin/env\tpython\n", true),
        case(b"#!/bin/sh\n", false)
    )]
    fn test_matches_shebang(pattern: &[u8], matches: bool) {
        let analyzer = Shebang::new(&["python", "python3"]);
        assert_eq!(analyzer.matches(pattern), matches);
    }
}
