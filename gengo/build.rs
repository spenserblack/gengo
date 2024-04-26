use indexmap::IndexMap;
use proc_macro2::{Ident, Literal, Span};
use quote::quote;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
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
    let languages_target_dir = Path::new(&env::var("OUT_DIR")?).join("languages");
    fs::create_dir_all(&languages_target_dir)?;
    let languages_target_path = Path::new(&env::var("OUT_DIR")?).join("language_generated.rs");

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

            let heuristics = language_attrs
                .get("heuristics")
                .map(|heuristics| {
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
                })
                .unwrap_or_default();

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
    let language = quote! {
        /// The type of language. Returned by language detection.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
        pub enum Language {
            #(#variants,)*
        }
    };
    fs::write(
        languages_target_dir.join("language.rs"),
        language.to_string(),
    )?;

    let category_mappings = language_definitions.iter().map(
        |LanguageDefinition {
             variant, category, ..
         }| {
            quote! {
                Self::#variant => Category::#category
            }
        },
    );
    let category_mixin = quote! {
        impl Language {
            /// Gets the category of the language.
            pub const fn category(&self) -> Category {
                match self {
                    #(#category_mappings ,)*
                }
            }
        }
    };
    fs::write(
        languages_target_dir.join("category_mixin.rs"),
        category_mixin.to_string(),
    )?;

    let name_mappings =
        language_definitions
            .iter()
            .map(|LanguageDefinition { variant, name, .. }| {
                quote! {
                    Self::#variant => #name
                }
            });
    let name_mixin = quote! {
        impl Language {
            /// Gets the name of the language.
            pub const fn name(&self) -> &'static str {
                match self {
                    #(#name_mappings ,)*
                }
            }
        }
    };
    fs::write(
        languages_target_dir.join("name_mixin.rs"),
        name_mixin.to_string(),
    )?;

    let reverse_variant_mappings =
        language_definitions
            .iter()
            .map(|LanguageDefinition { variant, .. }| {
                let variant_name = variant.to_string();
                quote! {
                    #variant_name => Some(Self::#variant)
                }
            });
    let parse_variant_mixin = quote! {
        impl Language {
            /// Converts a variant's name back to the language.
            fn parse_variant(name: &str) -> Option<Self> {
                match name {
                    #(#reverse_variant_mappings ,)*
                    _ => None,
                }
            }
        }
    };
    fs::write(
        languages_target_dir.join("parse_variant_mixin.rs"),
        parse_variant_mixin.to_string(),
    )?;

    let color_mappings =
        language_definitions
            .iter()
            .map(|LanguageDefinition { variant, color, .. }| {
                quote! {
                    Self::#variant => #color
                }
            });
    let color_mixin = quote! {
        impl Language {
            /// Gets the color associated with the language.
            pub const fn color(&self) -> &'static str {
                match self {
                    #(#color_mappings ,)*
                }
            }
        }
    };
    fs::write(
        languages_target_dir.join("color_mixin.rs"),
        color_mixin.to_string(),
    )?;

    let priority_mappings = language_definitions.iter().map(
        |LanguageDefinition {
             variant, priority, ..
         }| {
            quote! {
                Self::#variant => #priority
            }
        },
    );
    let priority_mixin = quote! {
        impl Language {
            /// Gets the priority of the language. This is useful for sorting languages
            /// when multiple languages are detected.
            pub const fn priority(&self) -> u8 {
                match self {
                    #(#priority_mappings ,)*
                }
            }
        }
    };
    fs::write(
        languages_target_dir.join("priority_mixin.rs"),
        priority_mixin.to_string(),
    )?;

    let extension_to_langs: HashMap<_, Vec<_>> = language_definitions.iter().fold(
        HashMap::new(),
        |map,
         LanguageDefinition {
             variant,
             extensions,
             ..
         }| {
            extensions.iter().fold(map, |mut map, extension| {
                map.entry(extension.clone())
                    .or_default()
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
    let from_extension_mixin = quote! {
        impl Language {
            /// Gets languages by extension.
            pub fn from_extension(extension: &str) -> Vec<Self> {
                match extension {
                    #(#extension_to_langs_mappings ,)*
                    _ => vec![],
                }
            }
        }
    };
    fs::write(
        languages_target_dir.join("from_extension_mixin.rs"),
        from_extension_mixin.to_string(),
    )?;

    let filenames_to_langs: HashMap<_, Vec<_>> = language_definitions.iter().fold(
        HashMap::new(),
        |map,
         LanguageDefinition {
             variant, filenames, ..
         }| {
            filenames.iter().fold(map, |mut map, filename| {
                map.entry(filename.clone())
                    .or_default()
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
    let from_filename_mixin = quote! {
        impl Language {
            /// Gets languages by filename.
            pub fn from_filename(filename: &str) -> Vec<Self> {
                match filename {
                    #(#filenames_to_langs_mappings ,)*
                    _ => vec![],
                }
            }
        }
    };
    fs::write(
        languages_target_dir.join("from_filename_mixin.rs"),
        from_filename_mixin.to_string(),
    )?;

    let interpreters_to_langs: HashMap<_, Vec<_>> = language_definitions.iter().fold(
        HashMap::new(),
        |map,
         LanguageDefinition {
             variant,
             interpreters,
             ..
         }| {
            interpreters.iter().fold(map, |mut map, interpreter| {
                map.entry(interpreter.clone())
                    .or_default()
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
    let from_interpreter_mixin = quote! {
        impl Language {
            /// Gets languages by interpreter (typically found as part of a shebang).
            pub fn from_interpreter(interpreter: &str) -> Vec<Self> {
                match interpreter {
                    #(#interpreter_to_langs_mappings ,)*
                    _ => vec![],
                }
            }
        }
    };
    fs::write(
        languages_target_dir.join("from_interpreter_mixin.rs"),
        from_interpreter_mixin.to_string(),
    )?;

    let glob_matchers = language_definitions
        .iter()
        .filter(|def| !def.patterns.is_empty())
        .map(
            |LanguageDefinition {
                 variant, patterns, ..
             }| {
                quote! {
                   (
                       vec![#(#patterns),*],
                       Language::#variant,
                   )
                }
            },
        );
    let glob_mappings_mixin = quote! {
        impl Language {
            /// Gets the mappings used to map a glob to its language.
            fn glob_mappings() -> Vec<(Vec<&'static str>, Self)> {
                vec![#(#glob_matchers),*]
            }
        }
    };
    fs::write(
        languages_target_dir.join("glob_mappings_mixin.rs"),
        glob_mappings_mixin.to_string(),
    )?;

    let heuristic_tuples = language_definitions
        .iter()
        .filter(|language_definition| !language_definition.heuristics.is_empty())
        .map(
            |LanguageDefinition {
                 variant,
                 heuristics,
                 ..
             }| {
                quote! {
                    (Self::#variant, vec![#(#heuristics),*])
                }
            },
        );
    let heuristic_mappings_mixin = quote! {
        impl Language {
            /// Gets the heuristics used to determine a language.
            fn heuristic_mappings() -> Vec<(Self, Vec<&'static str>)> {
                vec![#(#heuristic_tuples ,)*]
            }
        }
    };
    fs::write(
        languages_target_dir.join("heuristic_mappings_mixin.rs"),
        heuristic_mappings_mixin.to_string(),
    )?;

    let language_file_contents = quote! {
        use once_cell::sync::Lazy;
        use regex::Regex;
        use std::collections::HashMap;
        use std::path::Path;
        use crate::GLOB_MATCH_OPTIONS;

        impl Language {
            /// Uses simple checks to find one or more matching languages. Checks by shebang, filename,
            /// filepath glob, and extension.
            fn find_simple(path: impl AsRef<Path>, contents: &[u8]) -> Vec<Self> {
                let languages = Self::from_shebang(contents);
                if !languages.is_empty() {
                    return languages;
                }
                let languages = Self::from_path_filename(&path);
                if !languages.is_empty() {
                    return languages;
                }
                let languages = Self::from_glob(&path);
                if !languages.is_empty() {
                    return languages;
                }
                Self::from_path_extension(&path)
            }

            /// Picks the best guess from a file's name and contents.
            ///
            /// When checking heuristics, only the first `read_limit` bytes will be read.
            pub fn pick(path: impl AsRef<Path>, contents: &[u8], read_limit: usize) -> Option<Self> {
                let languages = Self::find_simple(&path, contents);
                if languages.len() == 1 {
                    return Some(languages[0]);
                }

                let contents = if contents.len() > read_limit {
                    &contents[..read_limit]
                } else {
                    contents
                };
                let heuristic_contents = std::str::from_utf8(contents).unwrap_or_default();
                let by_heuristics = Self::filter_by_heuristics(&languages, heuristic_contents);

                let found_languages = match by_heuristics.len() {
                    0 => languages,
                    1 => return Some(by_heuristics[0]),
                    _ => by_heuristics,
                };

                found_languages.into_iter().max_by_key(Self::priority)
            }
        }
    };

    let language_file_contents: syn::File = syn::parse2(language_file_contents).unwrap();
    let language_file_contents = prettyplease::unparse(&language_file_contents);

    fs::write(languages_target_path, dbg!(language_file_contents))?;
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
