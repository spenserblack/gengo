use std::{ffi::OsStr, path::MAIN_SEPARATOR};

pub struct Vendored;

impl Vendored {
    pub fn is_vendored(filepath: &OsStr, contents: &[u8]) -> bool {
        Self::is_vendored_no_read(filepath) || Self::is_vendored_with_read(filepath, contents)
    }

    fn is_vendored_no_read(filepath: &OsStr) -> bool {
        let prefix = format!("node_modules{}", MAIN_SEPARATOR);
        filepath.to_str().unwrap_or_default().starts_with(&prefix)
    }

    fn is_vendored_with_read(_filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }
}
