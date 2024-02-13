use indexmap::IndexMap;
use proc_macro2::{Ident, Literal, Span};
use quote::quote;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const LANGUAGES: &str = include_str!("./languages.yaml");

const MINIMUM_PRIORITY: u8 = 0;
const MAXIMUM_PRIORITY: u8 = 100;
const DEFAULT_PRIORITY: u8 = 50;

/// Converts `languages.yaml` to minified JSON and writes it to
/// `languages.json`.
fn main() -> Result<(), Box<dyn Error>> {
    // TODO This looks messy, and can use cleanup.
    let languages: IndexMap<String, serde_json::Value> = serde_yaml::from_str(LANGUAGES)?;
    let languages_target_path = Path::new(&env::var("OUT_DIR")?).join("languages_generated.rs");

    struct LanguageDefinition {
        variant: Ident,
        /// See `Category`.
        category: Ident,
        name: Literal,
        color: Literal,
        priority: Literal,
        extensions: Vec<String>,
        filenames: Vec<String>,
        interpreters: Vec<String>,
        patterns: Vec<String>,
        heuristics: Vec<String>,
    }

    let language_definitions: Vec<_> = languages
        .iter()
        .map(|(language_name, language_attrs)| {
            let language_attrs = language_attrs
                .as_object()
                .expect("language attributes to be an object");
            let variant = rustify_language_name(language_name);
            let variant = Ident::new(&variant, Span::call_site());

            let category = language_attrs["category"]
                .as_str()
                .expect("category to be a string");
            let category = match category {
                "data" => "Data",
                "markup" => "Markup",
                "programming" => "Programming",
                "prose" => "Prose",
                "query" => "Query",
                unknown => unreachable!("Category {unknown}"),
            };
            let category = Ident::new(category, Span::call_site());

            let name = Literal::string(language_name);

            let color = language_attrs["color"]
                .as_str()
                .expect("color to be a string");
            let color = Literal::string(color);

            let priority = language_attrs
                .get("priority")
                .map(|priority| {
                    let priority = priority.as_u64().expect("priority to be a number");
                    assert!(
                        priority >= MINIMUM_PRIORITY.into() && priority <= MAXIMUM_PRIORITY.into(),
                        "priority to be between {MINIMUM_PRIORITY} and {MAXIMUM_PRIORITY}"
                    );
                    priority as u8
                })
                .unwrap_or(DEFAULT_PRIORITY);
            let priority = Literal::u8_unsuffixed(priority);

            let matchers = language_attrs["matchers"]
                .as_object()
                .expect("matchers to be an object");

            let extensions = matchers
                .get("extensions")
                .map(|extensions| {
                    extensions
                        .as_array()
                        .expect("extensions to be an array")
                        .to_owned()
                })
                .unwrap_or_default()
                .iter()
                .map(|extension| {
                    extension
                        .as_str()
                        .expect("extension to be a string")
                        .to_string()
                })
                .collect();

            let filenames = matchers
                .get("filenames")
                .map(|filenames| {
                    filenames
                        .as_array()
                        .expect("filenames to be an array")
                        .to_owned()
                })
                .unwrap_or_default()
                .iter()
                .map(|filename| {
                    filename
                        .as_str()
                        .expect("filename to be a string")
                        .to_string()
                })
                .collect();

            let interpreters = matchers
                .get("interpreters")
                .map(|interpreters| {
                    interpreters
                        .as_array()
                        .expect("interpreters to be an array")
                        .to_owned()
                })
                .unwrap_or_default()
                .iter()
                .map(|interpreter| {
                    interpreter
                        .as_str()
                        .expect("interpreter to be a string")
                        .to_string()
                })
                .collect();

            let patterns = matchers
                .get("patterns")
                .map(|patterns| {
                    patterns
                        .as_array()
                        .expect("patterns to be an array")
                        .to_owned()
                })
                .unwrap_or_default()
                .iter()
                .map(|pattern| {
                    pattern
                        .as_str()
                        .expect("pattern to be a string")
                        .to_string()
                })
                .collect();

            let heuristics = language_attrs.get("heuristics").map(|heuristics| {
                heuristics
                    .as_array()
                    .expect("heuristics to be an array")
                    .to_owned()
                    .iter()
                    .map(|heuristic| {
                        heuristic
                            .as_str()
                            .expect("heuristic to be a string")
                            .to_string()
                    })
                    .collect()
            }).unwrap_or_default();

            LanguageDefinition {
                variant,
                category,
                name,
                color,
                priority,
                extensions,
                filenames,
                interpreters,
                patterns,
                heuristics,
            }
        })
        .collect();

    let variants = language_definitions.iter().map(|def| &def.variant);

    let category_mappings = language_definitions.iter().map(
        |LanguageDefinition {
             variant, category, ..
         }| {
            quote! {
                Self::#variant => Category::#category
            }
        },
    );

    let name_mappings =
        language_definitions
            .iter()
            .map(|LanguageDefinition { variant, name, .. }| {
                quote! {
                    Self::#variant => #name
                }
            });

    let color_mappings =
        language_definitions
            .iter()
            .map(|LanguageDefinition { variant, color, .. }| {
                quote! {
                    Self::#variant => #color
                }
            });

    let priority_mappings = language_definitions.iter().map(
        |LanguageDefinition {
             variant, priority, ..
         }| {
            quote! {
                Self::#variant => #priority
            }
        },
    );

    let extension_to_langs = language_definitions.iter().fold(
        HashMap::new(),
        |map,
         LanguageDefinition {
             variant,
             extensions,
             ..
         }| {
            extensions.iter().fold(map, |mut map, extension| {
                map.entry(extension.clone())
                    .or_insert_with(Vec::new)
                    .push(variant.clone());
                map
            })
        },
    );
    let extension_to_langs_mappings = extension_to_langs.iter().map(|(extension, langs)| {
        quote! {
            #extension => vec![#(Self::#langs),*]
        }
    });

    let filenames_to_langs = language_definitions.iter().fold(
        HashMap::new(),
        |map,
         LanguageDefinition {
             variant,
             filenames,
             ..
         }| {
            filenames.iter().fold(map, |mut map, filename| {
                map.entry(filename.clone())
                    .or_insert_with(Vec::new)
                    .push(variant.clone());
                map
            })
        },
    );

    let filenames_to_langs_mappings = filenames_to_langs.iter().map(|(filename, langs)| {
        quote! {
            #filename => vec![#(Self::#langs),*]
        }
    });

    let interpreters_to_langs = language_definitions.iter().fold(
        HashMap::new(),
        |map,
         LanguageDefinition {
             variant,
             interpreters,
             ..
         }| {
            interpreters.iter().fold(map, |mut map, interpreter| {
                map.entry(interpreter.clone())
                    .or_insert_with(Vec::new)
                    .push(variant.clone());
                map
            })
        },
    );

    let interpreter_to_langs_mappings = interpreters_to_langs.iter().map(|(interpreter, langs)| {
        quote! {
            #interpreter => vec![#(Self::#langs),*]
        }
    });

    let glob_matchers = language_definitions
        .iter()
        .filter(|def| !def.patterns.is_empty())
        .map(
            |LanguageDefinition {
                 variant, patterns, ..
             }| {
                quote! {
                   GlobMapping {
                       patterns: vec![#(glob::Pattern::new( #patterns ).unwrap()),*],
                       language: Language::#variant,
                   }
                }
            },
        );

    let heuristic_inserts = language_definitions.iter().filter(|language_definition| !language_definition.heuristics.is_empty()).map(|LanguageDefinition { variant, heuristics, ..}| {
        quote! {
            map.insert(Language::#variant, vec![#(regex::Regex::new(#heuristics).unwrap()),*]);
        }
    });

    let language_file_contents = quote! {
        use once_cell::sync::Lazy;
        use regex::Regex;
        use std::collections::HashMap;
        use std::path::Path;
        use crate::GLOB_MATCH_OPTIONS;

        /// The type of language. Returned by language detection.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
        pub enum Language {
            #(#variants,)*
        }

        impl Language {
            /// Gets the category of the language.
            pub const fn category(&self) -> Category {
                match self {
                    #(#category_mappings ,)*
                }
            }

            /// Gets the name of the language.
            pub const fn name(&self) -> &'static str {
                match self {
                    #(#name_mappings ,)*
                }
            }

            /// Gets the color associated with the language.
            pub const fn color(&self) -> &'static str {
                match self {
                    #(#color_mappings ,)*
                }
            }

            /// Gets the priority of the language. This is useful for sorting languages
            /// when multiple languages are detected.
            const fn priority(&self) -> u8 {
                match self {
                    #(#priority_mappings ,)*
                }
            }

            /// Gets languages by extension.
            pub fn from_extension(extension: &str) -> Vec<Self> {
                match extension {
                    #(#extension_to_langs_mappings ,)*
                    _ => vec![],
                }
            }

            /// Gets languages from a path's extension.
            fn from_path_extension(path: impl AsRef<Path>) -> Vec<Self> {
                let extension = path.as_ref().extension().and_then(|ext| ext.to_str());
                extension.map_or(vec![], Self::from_extension)
            }

            /// Gets languages by filename.
            pub fn from_filename(filename: &str) -> Vec<Self> {
                match filename {
                    #(#filenames_to_langs_mappings ,)*
                    _ => vec![],
                }
            }

            /// Gets languages from a path's filename.
            fn from_path_filename(path: impl AsRef<Path>) -> Vec<Self> {
                let filename = path.as_ref().file_name().and_then(|filename| filename.to_str());
                filename.map_or(vec![], Self::from_filename)
            }

            /// Gets languages by interpreter (typically found as part of a shebang).
            pub fn from_interpreter(interpreter: &str) -> Vec<Self> {
                match interpreter {
                    #(#interpreter_to_langs_mappings ,)*
                    _ => vec![],
                }
            }

            /// Gets languages by a shebang.
            fn from_shebang(contents: &[u8]) -> Vec<Self> {
                const MAX_SHEBANG_LENGTH: usize = 50;

                let mut lines = contents.split(|&c| c == b'\n');
                let first_line = lines.next().unwrap_or_default();
                if first_line.len() < 2 || first_line[0] != b'#' || first_line[1] != b'!' {
                    return vec![];
                }
                let first_line = if first_line.len() > MAX_SHEBANG_LENGTH {
                    &first_line[..MAX_SHEBANG_LENGTH]
                } else {
                    first_line
                };
                let first_line = String::from_utf8_lossy(first_line);
                // NOTE Handle trailing spaces, `\r`, etc.
                let first_line = first_line.trim_end();

                static RE: Lazy<Regex> = Lazy::new(|| {
                    Regex::new(r"^#!(?:/usr(?:/local)?)?/bin/(?:env\s+)?([\w\d]+)\r?$").unwrap()
                });

                RE.captures(first_line)
                    .and_then(|c| c.get(1))
                    .map_or(vec![], |m| {
                        let interpreter = m.as_str();
                        Self::from_interpreter(interpreter)
                    })
            }

            /// Gets the languages that match a glob pattern.
            pub fn from_glob(path: impl AsRef<Path>) -> Vec<Self> {
                let path = path.as_ref();

                struct GlobMapping {
                    patterns: Vec<glob::Pattern>,
                    language: Language,
                }
                static GLOB_MAPPINGS: Lazy<Vec<GlobMapping>> = Lazy::new(|| {
                    vec![#(#glob_matchers),*]
                });

                GLOB_MAPPINGS
                    .iter()
                    .filter(|gm| gm.patterns.iter().any(|p| p.matches_path_with(path.as_ref(), GLOB_MATCH_OPTIONS)))
                    .map(|gm| gm.language)
                    .collect()
            }

            /// Filters an iterable of languages by heuristics.
            fn filter_by_heuristics(languages: &[Self], contents: &str) -> Vec<Self> {
                static HEURISTICS: Lazy<HashMap<Language, Vec<Regex>>> = Lazy::new(|| {
                    let mut map = HashMap::new();
                    #(#heuristic_inserts)*
                    map
                });

                languages
                    .iter()
                    .filter(|language| {
                        HEURISTICS
                            .get(language)
                            .map_or(false, |heuristics| heuristics.iter().any(|re| re.is_match(contents)))
                    })
                    .cloned()
                    .collect()
            }
        }
    };

    let language_file_contents: syn::File = syn::parse2(language_file_contents).unwrap();
    let language_file_contents = prettyplease::unparse(&language_file_contents);

    fs::write(languages_target_path, dbg!(language_file_contents))?;
    // panic!("force debug output");

    // ----------- Old way -------------

    let languages_target_path = Path::new(&env::var("OUT_DIR")?).join("languages.json");
    let json = serde_json::to_string(&languages)?;
    fs::write(languages_target_path, json)?;

    let doc_target_path = Path::new(&env::var("OUT_DIR")?).join("language-list.md");
    let mut doc_file = File::create(doc_target_path)?;
    for language in languages.keys() {
        writeln!(doc_file, "- {}", language)?;
    }

    Ok(())
}

/// Converts a language name to a valid Rust identifier to be used as an enum
/// variant.
///
/// # Examples
///
/// - `".Env"` -> `"Dotenv"`
/// - `"C++"` -> `"CPlusPlus"`
/// - `"C#"` -> `"CSharp"`
/// - `"HTML"` -> `"Html"`
/// - `"JavaScript"` -> `"Javascript"`
/// - `"Batch File"` -> `"BatchFile"`
fn rustify_language_name(name: &str) -> String {
    let name = asciiify_language_name(name);

    // HACK This will break if there are any leading, trailing, or consecutive
    //      spaces in the name.
    let name = name.split(' ').fold(String::new(), |name, word| {
        let mut chars = word.chars();
        // NOTE If there is a special character like ß it will become SS, but
        //      that should never happen.
        let first_char = chars.next().unwrap().to_uppercase();
        assert!(first_char.len() == 1);
        let rest = chars
            .map(|c| c.to_lowercase().to_string())
            .collect::<String>();
        format!("{name}{first_char}{rest}")
    });
    name
}

/// Replaces special characters in a language name with their ASCII
/// equivalents.
fn asciiify_language_name(name: &str) -> String {
    // NOTE .Env is a special case because the special character is at the beginning
    //      and it should be one word.
    if name == ".Env" {
        return "Dotenv".to_string();
    }
    // NOTE Maps special characters to their ASCII equivalents.
    let mappings = [("-", ""), ("'", ""), ("+", "Plus"), ("#", "Sharp")];

    let name = mappings
        .iter()
        .fold(name.to_string(), |name, (pattern, replacement)| {
            // NOTE Adding a leading space to the replacement to ensure that it
            //      is treated as a word boundary.
            name.replace(pattern, &format!(" {replacement}"))
        });

    name
}
