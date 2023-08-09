use std::ffi::OsStr;

pub struct Documentation;

impl Documentation {
    pub fn is_documentation(filepath: &OsStr, contents: &[u8]) -> bool {
        Self::is_documentation_no_read(filepath)
            || Self::is_documentation_with_read(filepath, contents)
    }

    fn is_documentation_no_read(filepath: &OsStr) -> bool {
        filepath.to_str().unwrap_or_default().starts_with("docs/")
    }

    fn is_documentation_with_read(_filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }
}
