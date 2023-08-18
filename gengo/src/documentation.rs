use std::path::Path;

pub struct Documentation;

impl Documentation {
    pub fn new() -> Self {
        Self
    }

    pub fn is_documentation<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.is_documentation_no_read(&filepath)
            || self.is_documentation_with_read(&filepath, contents)
    }

    fn is_documentation_no_read<P: AsRef<Path>>(&self, filepath: P) -> bool {
        filepath
            .as_ref()
            .components()
            .next()
            .map_or(false, |c| c.as_os_str() == "docs")
    }

    fn is_documentation_with_read<P: AsRef<Path>>(&self, _filepath: P, _contents: &[u8]) -> bool {
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
        let documentation = Documentation::new();
        assert_eq!(documentation.is_documentation_no_read(filepath), expected);
    }
}
