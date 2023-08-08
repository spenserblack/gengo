use std::process::exit;
fn main() {
    let cli = gengo_bin::cli::new();
    if let Err(err) = cli.run(std::io::stdout(), std::io::stderr()) {
        eprintln!("{}", err);
        exit(1);
    }
}
