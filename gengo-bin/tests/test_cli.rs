macro_rules! assert_stdout {
    ($args:expr, $stdout:expr $(,)?) => {
        {
            let cli = gengo_bin::cli::new_from($args);
            let mut buf = Vec::new();
            cli.run(&mut buf).unwrap();
            let output = String::from_utf8(buf).unwrap();
            assert_eq!(output, $stdout);
        }
    }
}

#[test]
fn it_works() {
    assert_stdout!(&["gengo", "1", "2"], "1 + 2 = 3\n")
}
