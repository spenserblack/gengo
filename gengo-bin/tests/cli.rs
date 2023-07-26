#[test]
fn it_works() {
    let cli = gengo_bin::cli::new();
    let mut w = String::new();
    match cli.run(&mut w) {
        Ok(_) => assert_eq!(w, "2 + 2 = 4\n"),
        Err(_) => assert!(false),
    }
}
