use std::path::Path;

pub struct Documentation;

impl Documentation {
    pub fn is_documentation<P: AsRef<Path>>(filepath: P, contents: &[u8]) -> bool {
        Self::is_documentation_no_read(&filepath)
            || Self::is_documentation_with_read(&filepath, contents)
    }

    fn is_documentation_no_read<P: AsRef<Path>>(filepath: P) -> bool {
        filepath
            .as_ref()
            .components()
            .next()
            .map_or(false, |c| c.as_os_str() == "docs")
    }

    fn is_documentation_with_read<P: AsRef<Path>>(_filepath: P, _contents: &[u8]) -> bool {
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
        assert_eq!(Documentation::is_documentation_no_read(filepath), expected);
    }
}
