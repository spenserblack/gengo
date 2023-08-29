use indexmap::IndexMap;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const LANGUAGES: &str = include_str!("./languages.yaml");

/// Converts `languages.yaml` to minified JSON and writes it to
/// `languages.json`.
fn main() -> Result<(), Box<dyn Error>> {
    let languages: IndexMap<String, serde_json::Value> = serde_yaml::from_str(LANGUAGES)?;

    let languages_target_path = Path::new(&env::var("OUT_DIR")?).join("languages.json");
    let json = serde_json::to_string(&languages)?;
    fs::write(languages_target_path, json)?;

    let doc_target_path = Path::new(&env::var("OUT_DIR")?).join("language-list.md");
    let mut doc_file = File::create(&doc_target_path)?;
    for language in languages.keys() {
        writeln!(doc_file, "- {}", language)?;
    }

    Ok(())
}
