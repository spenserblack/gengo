use std::path::Path;

pub struct Generated;

impl Generated {
    pub fn new() -> Self {
        Self
    }

    pub fn is_generated<P: AsRef<Path>>(&self, filepath: P, contents: &[u8]) -> bool {
        self.is_generated_no_read(&filepath) || self.is_generated_with_read(&filepath, contents)
    }

    fn is_generated_no_read<P: AsRef<Path>>(&self, filepath: P) -> bool {
        filepath
            .as_ref()
            .components()
            .next()
            .map_or(false, |c| c.as_os_str() == "dist")
    }

    fn is_generated_with_read<P: AsRef<Path>>(&self, _filepath: P, _contents: &[u8]) -> bool {
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
        case("dist/subfolder/something.js", true)
    )]
    fn test_is_generated_no_read(filepath: &str, expected: bool) {
        let generated = Generated::new();
        assert_eq!(generated.is_generated_no_read(filepath), expected);
    }
}
