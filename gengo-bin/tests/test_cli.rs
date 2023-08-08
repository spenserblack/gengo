use insta::assert_snapshot;

macro_rules! assert_stdout_snapshot {
    ($cli_args:expr $(,)?) => {{
        let cli = gengo_bin::cli::try_new_from($cli_args).unwrap();
        let mut buf = Vec::new();
        cli.run(&mut buf).unwrap();
        let stdout = String::from_utf8(buf).unwrap();
        assert_snapshot!(stdout);
    }};
}

#[test]
fn it_works() {
    assert_stdout_snapshot!(&["gengo"]);
}
