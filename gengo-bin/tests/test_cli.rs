use insta::assert_snapshot;
use std::io::{self, Write};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

struct NullWriter;

impl Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

macro_rules! assert_stdout_snapshot {
    ($cli_args:expr $(,)?) => {{
        let cli = gengo_bin::cli::try_new_from($cli_args).unwrap();
        let mut stdout = Vec::new();
        let mut stderr = NullWriter;
        cli.run(&mut stdout, &mut stderr).unwrap();
        let stdout = String::from_utf8(stdout).unwrap();
        assert_snapshot!(stdout);
    }};
}

#[test]
fn test_javascript_repo() {
    assert_stdout_snapshot!(&["gengo", "-r", "test/javascript", "-R", ROOT]);
}

#[test]
#[cfg_attr(windows, ignore)]
fn test_breakdown_javascript_repo() {
    assert_stdout_snapshot!(&["gengo", "-r", "test/javascript", "-R", ROOT, "--breakdown"]);
}

//
// TODO Add test_javascript_repo_windows
