use std::ffi::OsStr;

pub struct Vendored;

impl Vendored {
    pub fn is_vendored(filepath: &OsStr, contents: &[u8]) -> bool {
        Self::is_vendored_no_read(filepath) || Self::is_vendored_with_read(filepath, contents)
    }

    fn is_vendored_no_read(filepath: &OsStr) -> bool {
        filepath
            .to_str()
            .unwrap_or_default()
            .starts_with("node_modules/")
    }

    fn is_vendored_with_read(_filepath: &OsStr, _contents: &[u8]) -> bool {
        false
    }
}
