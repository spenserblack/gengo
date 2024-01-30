use std::path::Path;

pub struct Binary {
    read_limit: usize,
}

impl Binary {
    pub fn new(read_limit: usize) -> Self {
        Self { read_limit }
    }

    /// Guesses if a file is binary.
    pub fn is_binary(&self, filepath: impl AsRef<Path>, contents: &[u8]) -> bool {
        self.is_binary_no_read(&filepath) || self.is_binary_with_read(&filepath, contents)
    }

    fn is_binary_no_read(&self, _filepath: impl AsRef<Path>) -> bool {
        false
    }

    fn is_binary_with_read(&self, _filepath: impl AsRef<Path>, contents: &[u8]) -> bool {
        // NOTE This is currently very naive, and will need to be improved as issues
        //      are found.

        // TODO Simply return `false` if `contents.len()` is less than the read limit?
        //      This would be because exceptionally large files could be binary.

        // NOTE If any of the bytes is a null byte, this is likely binary.
        contents.iter().take(self.read_limit).any(|&b| b == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        filepath,
        contents,
        expected,
        case("foo.txt", "", false),
        case("foo.txt", "Hello, world!", false),
        case("foo.txt", "ab\0c", true)
    )]
    fn test_is_binary_with_read(filepath: &str, contents: &str, expected: bool) {
        let binary = Binary::new(1 << 20);
        assert_eq!(
            binary.is_binary_with_read(filepath, contents.as_bytes()),
            expected
        );
    }
}
