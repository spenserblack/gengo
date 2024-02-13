use gengo::Language;
use once_cell::sync::Lazy;
use std::collections::HashMap;

const LANGUAGES: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/languages.yaml"));
static LANGUAGES_MAP: Lazy<HashMap<String, serde_yaml::Value>> =
    Lazy::new(|| serde_yaml::from_str(LANGUAGES).unwrap());

fn get_matchers(name: &str) -> Vec<(String, Vec<String>)> {
    LANGUAGES_MAP
        .iter()
        .map(|(language_name, attrs)| {
            let map = attrs.as_mapping().unwrap();
            let matchers = map["matchers"].as_mapping().unwrap();
            let matchers = matchers
                .get(name)
                .map(|matchers| matchers.as_sequence().unwrap())
                .map(|matchers| {
                    matchers
                        .iter()
                        .map(|matcher| matcher.as_str().unwrap().to_string())
                        .collect()
                })
                .unwrap_or_default();
            (language_name.to_string(), matchers)
        })
        .collect()
}

#[test]
fn test_from_extension() {
    let extensions = get_matchers("extensions");
    extensions.iter().for_each(|(language_name, extensions)| {
        extensions.iter().for_each(|extension| {
            let languages = Language::from_extension(extension);
            let languages: Vec<_> = languages.iter().map(|language| language.name()).collect();

            assert!(
                languages.contains(&language_name.as_str()),
                "Language::from_extension({extension}) contains {language_name}"
            );
        });
    });
}

#[test]
fn test_unused_extension() {
    let languages = Language::from_extension("totally.unused.extension");
    assert!(languages.is_empty());
}

#[test]
fn test_from_filename() {
    let filenames = get_matchers("filenames");
    filenames.iter().for_each(|(language_name, filenames)| {
        filenames.iter().for_each(|filename| {
            let languages = Language::from_filename(filename);
            let languages: Vec<_> = languages.iter().map(|language| language.name()).collect();

            assert!(
                languages.contains(&language_name.as_str()),
                "Language::from_filename({filename}) contains {language_name}"
            );
        });
    });
}

#[test]
fn test_unused_filename() {
    let languages = Language::from_filename("!!!totally-unused-filename!!!");
    assert!(languages.is_empty());
}

#[test]
fn test_from_interpreter() {
    let interpreters = get_matchers("interpreters");
    interpreters
        .iter()
        .for_each(|(language_name, interpreters)| {
            interpreters.iter().for_each(|interpreter| {
                let languages = Language::from_interpreter(interpreter);
                let languages: Vec<_> = languages.iter().map(|language| language.name()).collect();

                assert!(
                    languages.contains(&language_name.as_str()),
                    "Language::from_interpreter({interpreter}) contains {language_name}"
                );
            });
        });
}

#[test]
fn test_unused_interpreter() {
    let languages = Language::from_interpreter(".totally-unused-interpreter");
    assert!(languages.is_empty());
}
