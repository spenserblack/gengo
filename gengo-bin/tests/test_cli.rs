use insta::{assert_json_snapshot, assert_snapshot};
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

// TODO This feels like a bit of a hack.
macro_rules! assert_stdout_json_snapshot {
    ($cli_args:expr $(,)?) => {{
        let cli = gengo_bin::cli::try_new_from($cli_args).unwrap();
        let mut stdout = Vec::new();
        let mut stderr = NullWriter;
        cli.run(&mut stdout, &mut stderr).unwrap();
        let stdout = String::from_utf8(stdout).unwrap();
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_json_snapshot!(json);
    }};
}

#[test]
#[cfg(not(feature = "color"))]
fn test_javascript_repo() {
    assert_stdout_snapshot!(&["gengo", "git", "-r", "test/javascript", "-R", ROOT]);
}

#[test]
#[cfg_attr(windows, ignore)]
#[cfg(not(feature = "color"))]
fn test_breakdown_javascript_repo() {
    assert_stdout_snapshot!(&[
        "gengo",
        "--breakdown",
        "git",
        "-r",
        "test/javascript",
        "-R",
        ROOT,
    ]);
}

#[test]
#[cfg(feature = "color")]
fn test_color_javascript_repo() {
    assert_stdout_snapshot!(&["gengo", "git", "-r", "test/javascript", "-R", ROOT]);
}

#[test]
#[cfg_attr(windows, ignore)]
#[cfg(feature = "color")]
fn test_color_breakdown_javascript_repo() {
    assert_stdout_snapshot!(&[
        "gengo",
        "--breakdown",
        "git",
        "-r",
        "test/javascript",
        "-R",
        ROOT,
    ]);
}

// TODO Add test_javascript_repo_windows

#[test]
#[cfg_attr(windows, ignore)]
fn test_json_output_on_javascript_repo() {
    assert_stdout_json_snapshot!(&[
        "gengo",
        "--format",
        "json",
        "git",
        "-r",
        "test/javascript",
        "-R",
        ROOT,
    ]);
}
