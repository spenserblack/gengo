use std::ffi::OsStr;

pub struct Generated;

impl Generated {
    pub fn is_generated(filepath: &OsStr, contents: &[u8]) -> bool {
        Self::is_generated_no_read(filepath) || Self::is_generated_with_read(filepath, contents)
    }

    fn is_generated_no_read(filepath: &OsStr) -> bool {
        filepath.to_str().unwrap_or_default().starts_with("dist/")
    }

    fn is_generated_with_read(_filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }
}
