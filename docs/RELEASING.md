# Releasing

Releasing is a simple process -- just create a new GitHub release. Use the
auto-generate release notes feature, labelling PRs if necessary. The workflows should
take care of releasing to crates.io and making release assets.

## Versioning

### Major (breaking) changes to languages

These may not be obvious at first, as these are changes made to the data, not the Rust
code. The following changes to `languages.yaml` are considered breaking:

* Renaming a language
* Removing a language
* Removing a matcher (extension, interpreter, glob)[^1]

[^1]: An exception to this is if the matcher is so rarely used that it shouldn't have been added in the first place.
