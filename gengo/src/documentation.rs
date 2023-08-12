use std::{ffi::OsStr, path::MAIN_SEPARATOR};

pub struct Documentation;

impl Documentation {
    pub fn is_documentation(filepath: &OsStr, contents: &[u8]) -> bool {
        Self::is_documentation_no_read(filepath)
            || Self::is_documentation_with_read(filepath, contents)
    }

    fn is_documentation_no_read(filepath: &OsStr) -> bool {
        let prefix = format!("docs{}", MAIN_SEPARATOR);
        filepath.to_str().unwrap_or_default().starts_with(&prefix)
    }

    fn is_documentation_with_read(_filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }
}
