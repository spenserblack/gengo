pub use gengo;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt::{self, Display};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug)]
    struct TestError {
        expected_language: gengo::LanguageOld,
        actual_language: gengo::LanguageOld,
        filename: PathBuf,
    }

    impl Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Expected {} to be {}, got {}",
                self.filename.display(),
                self.expected_language.name(),
                self.actual_language.name()
            )
        }
    }

    impl Error for TestError {}

    #[test]
    fn test_samples() -> Result<(), Vec<TestError>> {
        let analyzers = gengo::Analyzers::default();
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let samples_dir = manifest_dir.join("samples");
        let errors: Vec<_> = samples_dir
            .read_dir()
            .expect("samples directory to be readable")
            .flat_map(|entry| {
                let path = entry.expect("entry to be readable").path();
                let language_dir = path.strip_prefix(&samples_dir).unwrap();
                let language_name = language_dir.display().to_string();
                let expected = analyzers.get(&language_name).expect("language to exist");
                path.read_dir()
                    .expect("language directory to be readable")
                    .map(|entry| {
                        let path = entry.expect("entry to be readable").path();
                        let contents = fs::read(&path).expect("file to be readable");
                        let relative_path = path.strip_prefix(&samples_dir).unwrap();
                        let actual = analyzers
                            .pick(relative_path, &contents, 1 << 20)
                            .expect("language to exist");
                        if expected.name() != actual.name() {
                            Err(TestError {
                                expected_language: expected.clone(),
                                actual_language: actual.clone(),
                                filename: relative_path.to_owned(),
                            })
                        } else {
                            Ok(())
                        }
                    })
            })
            .filter_map(Result::err)
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
