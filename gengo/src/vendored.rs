use std::path::Path;

pub struct Vendored;

impl Vendored {
    pub fn new() -> Self {
        Self
    }

    pub fn is_vendored<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.is_vendored_no_read(&filepath) || self.is_vendored_with_read(&filepath, contents)
    }

    fn is_vendored_no_read<P: AsRef<Path>>(&self, filepath: P) -> bool {
        filepath
            .as_ref()
            .components()
            .next()
            .map_or(false, |c| c.as_os_str() == "node_modules")
    }

    fn is_vendored_with_read<P: AsRef<Path>>(&self, _filepath: P, _contents: &[u8]) -> bool {
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
        case("node_modules/library/something.js", true),
        case("src/something.rs", false),
        case("node_modules/subfolder/something.js", true),
        case("", false),
        case("node_modules", true)
    )]
    fn test_is_vendored_no_read(filepath: &str, expected: bool) {
        let vendored = Vendored::new();
        assert_eq!(vendored.is_vendored_no_read(filepath), expected);
    }
}
