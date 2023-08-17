use std::path::Path;

pub struct Generated;

impl Generated {
    pub fn is_generated<P: AsRef<Path>>(filepath: P, contents: &[u8]) -> bool {
        Self::is_generated_no_read(&filepath) || Self::is_generated_with_read(&filepath, contents)
    }

    fn is_generated_no_read<P: AsRef<Path>>(filepath: P) -> bool {
        filepath
            .as_ref()
            .components()
            .next()
            .map_or(false, |c| c.as_os_str() == "dist")
    }

    fn is_generated_with_read<P: AsRef<Path>>(_filepath: P, _contents: &[u8]) -> bool {
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
        case("dist/something.js", true),
        case("src/something.rs", false),
        case("dist/subfolder/something.js", true),
        case("", false),
        case("dist", true)
    )]
    fn test_is_generated_no_read(filepath: &str, expected: bool) {
        assert_eq!(Generated::is_generated_no_read(filepath), expected);
    }
}
