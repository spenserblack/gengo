use std::env;
use std::error::Error;
use std::collections::HashMap;
use std::path::Path;
use tera::{Tera, Context, Value};
use serde::{Deserialize, Serialize};
use std::fs;

const LANGUAGES: &'static str = include_str!("./languages.yaml");

#[derive(Deserialize, Serialize)]
struct Language {
    color: String,
    matchers: Matchers,
    category: LanguageCategory,
}

#[derive(Deserialize, Serialize)]
struct Matchers {
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    filenames: Vec<String>,
    #[serde(default)]
    patterns: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum LanguageCategory {
    Data,
    Markup,
    Programming,
    Prose,
}

macro_rules! template {
    ($name:literal) => {
        include_str!(concat!("./templates/", $name, ".tera"))
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut tera = Tera::default();
    tera.register_filter("rustify", rustify);
    let languages: Value = serde_yaml::from_str(LANGUAGES)?;
    let mut context = Context::new();
    context.insert("languages", &languages);

    let languages_target_path = Path::new(&env::var("OUT_DIR")?).join("languages.rs");
    let code = tera.render_str(template!("languages.rs"), &context)?;
    fs::write(&languages_target_path, code)?;
    Ok(())
}

/// Takes a human readable string like `"Foo Bar"` and returns a Rust identifier like `FooBar`.
fn rustify(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let value = match value {
        Value::String(s) => s,
        _ => return Err("rustify filter only accepts strings".into()),
    };
    let rustified = value.replace(" ", "");
    Ok(Value::String(rustified))
}
