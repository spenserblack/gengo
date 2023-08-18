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

    fn is_generated_with_read<P: AsRef<Path>>(&self, _filepath: P, contents: &[u8]) -> bool {
        self.likely_minified(contents)
    }

    fn likely_minified(&self, contents: &[u8]) -> bool {
        // NOTE If the first 10 lines are really long, it's probably minified.
        contents.split(|&b| b == b'\n').take(10).any(|line| line.len() > 250)
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
        let generated = Generated::new();
        assert_eq!(generated.is_generated_no_read(filepath), expected);
    }

    #[test]
    fn test_likely_minified() {
        let generated = Generated::new();
        let header: Vec<u8> = b"/*!\n  * This is my license etc etc\n */".into_iter()
        .map(|&b| b)
        .collect();
        let contents = b"console.log('hello, world!');".repeat(50);
        let contents = [header, contents].concat();
        assert!(generated.likely_minified(&contents));
    }
}
