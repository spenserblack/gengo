//! Crate for reading the `languages.yaml` file to generate code.
//!
//! You probably don't want to use this.
use indexmap::IndexMap;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{parse_macro_input, LitStr};

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
    let languages = parse_macro_input!(input as LitStr);
    let tokens = language_enum_impl(&languages.value());
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
}
