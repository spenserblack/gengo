//! Crate for reading the `languages.yaml` file to generate code.
//!
//! You probably don't want to use this.
use indexmap::IndexMap;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Literal, Span};
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{parse_macro_input, LitStr};

const LANGUAGES_YAML: &str = include_str!("../languages.yaml");

type Languages = IndexMap<String, Language>;

#[derive(Debug, Deserialize, Serialize)]
struct Language {
    category: LanguageCategory,
    color: String,
    matchers: Matchers,
    #[serde(default)]
    heuristics: Vec<String>,
    #[serde(default = "default_priority")]
    priority: f32,
}

fn default_priority() -> f32 {
    0.5
}

#[derive(Debug, Deserialize, Serialize)]
struct Matchers {
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    filenames: Vec<String>,
    #[serde(default)]
    patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum LanguageCategory {
    Data,
    Markup,
    Programming,
    Prose,
}

fn rustify(s: &str) -> String {
    s.replace(' ', "")
}

fn language_enum_impl(languages: &str) -> TokenStream2 {
    let languages: Languages = serde_yaml::from_str(languages).unwrap();
    let variants: Vec<TokenStream2> = languages
        .iter()
        .map(|(name, _)| rustify(name))
        .map(|name| {
            let variant = Ident::new(&name, Span::call_site());
            quote! { #variant, }
        })
        .collect();
    quote! {
        enum Language {
            #(#variants)*
        }
    }
}

#[proc_macro]
pub fn language_enum(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        panic!("language_enum takes no arguments");
    }
    let tokens = language_enum_impl(LANGUAGES_YAML);
    tokens.into()
}

fn default_analyzers_impl(languages: &str) -> TokenStream2 {
    let languages: Languages = serde_yaml::from_str(languages).unwrap();
    let initializers: Vec<TokenStream2> = languages
        .iter()
        .map(|(name, language)| {
            let name = rustify(name);
            let name = Ident::new(&name, Span::call_site());
            let name = quote! { Language::#name };

            let category = match language.category {
                LanguageCategory::Data => "Data",
                LanguageCategory::Markup => "Markup",
                LanguageCategory::Programming => "Programming",
                LanguageCategory::Prose => "Prose",
            };
            let category = Ident::new(category, Span::call_site());
            let category = quote! { Category::#category };

            let color = Literal::string(&language.color);

            let extensions: Vec<TokenStream2> = language
                .matchers
                .extensions
                .iter()
                .map(|ext| {
                    Literal::string(ext);
                    quote! { #ext }
                })
                .collect();
            let extensions = quote! { &[#(#extensions ,)*] };

            let filenames: Vec<TokenStream2> = language
                .matchers
                .filenames
                .iter()
                .map(|filename| {
                    Literal::string(filename);
                    quote! { #filename }
                })
                .collect();
            let filenames = quote! { &[#(#filenames ,)*] };

            let patterns: Vec<TokenStream2> = language
                .matchers
                .patterns
                .iter()
                .map(|pattern| {
                    Literal::string(pattern);
                    quote! { #pattern }
                })
                .collect();
            let patterns = quote! { &[#(#patterns ,)*] };

            let heuristics: Vec<TokenStream2> = language
                .heuristics
                .iter()
                .map(|heuristic| {
                    Literal::string(heuristic);
                    quote! { #heuristic }
                })
                .collect();
            let heuristics = quote! { &[#(#heuristics ,)*] };

            let priority = Literal::f32_suffixed(language.priority);

            quote! {
                Analyzer::new(
                    #name,
                    #category,
                    #color,
                    #extensions,
                    #filenames,
                    #patterns,
                    #heuristics,
                    #priority,
                )
            }
        })
        .collect();
    quote! {
        vec![
            #(#initializers ,)*
        ]
    }
}

#[proc_macro]
pub fn default_analyzers(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        panic!("default_analyzers takes no arguments");
    }
    let tokens = default_analyzers_impl(LANGUAGES_YAML);
    tokens.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_debug_snapshot, assert_snapshot};
    use paste::paste;

    macro_rules! fixture {
        ($name:literal $(,)?) => {
            include_str!(concat!("../fixtures/", $name, ".yaml"))
        };
    }

    macro_rules! snapshot_test_macro_impl {
        ($name:ident , $fixture:literal $(,)?) => {
            paste! {
                #[test]
                fn [< $name _rendered >]() {
                    let tokens = $name(fixture!($fixture));
                    assert_snapshot!(tokens.to_string());
                }

                #[test]
                fn [< $name _tokens >]() {
                    let tokens = $name(fixture!($fixture));
                    assert_debug_snapshot!(tokens);
                }
            }
        };
    }

    snapshot_test_macro_impl!(language_enum_impl, "language_enum");
    snapshot_test_macro_impl!(default_analyzers_impl, "default_analyzers");
}
