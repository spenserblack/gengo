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
        expected_language_name: String,
        actual_language_name: String,
        filename: PathBuf,
    }

    impl Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Expected {} to be {}, got {}",
                self.filename.display(),
                self.expected_language_name,
                self.actual_language_name
            )
        }
    }

    impl Error for TestError {}

    #[test]
    fn test_samples() -> Result<(), Vec<TestError>> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let samples_dir = manifest_dir.join("samples");
        let errors: Vec<_> = samples_dir
            .read_dir()
            .expect("samples directory to be readable")
            .flat_map(|entry| {
                let samples_dir = samples_dir.clone();
                let path = entry.expect("entry to be readable").path();
                let language_dir = path.strip_prefix(&samples_dir).unwrap();
                path.read_dir()
                    .expect("language directory to be readable")
                    .map(|entry| {
                        let path = entry.expect("entry to be readable").path();
                        let contents = fs::read(&path).expect("file to be readable");
                        let relative_path = path.strip_prefix(&samples_dir).unwrap();
                        let actual = gengo::Language::pick(relative_path, &contents, 1 << 20)
                            .expect("language to exist");
                        let actual_language_name = actual.name().to_string();
                        let expected_language_name = language_dir.display().to_string();
                        if actual_language_name != expected_language_name {
                            Err(TestError {
                                expected_language_name,
                                actual_language_name,
                                filename: relative_path.to_owned(),
                            })
                        } else {
                            Ok(())
                        }
                    })
                    .collect::<Vec<_>>()
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
