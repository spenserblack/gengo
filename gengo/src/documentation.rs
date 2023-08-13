use std::{ffi::OsStr, path::Path};

pub struct Documentation;

impl Documentation {
    pub fn is_documentation(filepath: &OsStr, contents: &[u8]) -> bool {
        Self::is_documentation_no_read(filepath)
            || Self::is_documentation_with_read(filepath, contents)
    }

    fn is_documentation_no_read(filepath: &OsStr) -> bool {
        Path::new(filepath)
            .components()
            .next()
            .map_or(false, |c| c.as_os_str() == "docs")
    }

    fn is_documentation_with_read(_filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        filepath,
        expected,
        case("docs/something.md", true),
        case("src/something.rs", false),
        case("docs/subfolder/something.md", true),
        case("", false),
        case("docs", true)
    )]
    fn test_is_documentation_no_read(filepath: &str, expected: bool) {
        assert_eq!(
            Documentation::is_documentation_no_read(OsStr::new(filepath)),
            expected
        );
    }
}
