use std::{ffi::OsStr, path::Path};

pub struct Vendored;

impl Vendored {
    pub fn is_vendored(filepath: &OsStr, contents: &[u8]) -> bool {
        Self::is_vendored_no_read(filepath) || Self::is_vendored_with_read(filepath, contents)
    }

    fn is_vendored_no_read(filepath: &OsStr) -> bool {
        Path::new(filepath)
            .components()
            .next()
            .map_or(false, |c| c.as_os_str() == "node_modules")
    }

    fn is_vendored_with_read(_filepath: &OsStr, _contents: &[u8]) -> bool {
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
        assert_eq!(
            Vendored::is_vendored_no_read(OsStr::new(filepath)),
            expected
        );
    }
}
