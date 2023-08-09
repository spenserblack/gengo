# Hacking

## Getting Started

### Tools

#### Testing

This project uses [insta][insta] for snapshot testing. Review the instructions
for using insta and installing the `cargo insta` executable.

## Language Support

While the library does support providing your own language definitions, the
"out-of-the-box" language definitions are defined in
[`./gengo/languages.yaml`][languages-file]. This is where you register
new languages, add extensions, filenames, filepath patterns, or interpreters
to help identify languages, etc. Pretty much anything involving language
detection or the returned language data comes from this file (but there are
a few exceptions).

### The `languages.yaml` file

A language looks roughly like this:

```yaml
My Language:
  category: programming
  color: "#AABBCC"
  heuristics:
    - "(?m)^my super unique syntax"
  matchers:
    extensions:
      - ext
    filenames:
      - Mylang
    interpreters:
      - mylangexe
    patterns:
      - "(?i)(^|/)special-path/"
  priority: 100
```

- `category` - Required. It should be `data`, `markup`, `programming`, or `prose`.
- `color` - Required. It should be a hexadecimal color. Since `#` is a comment
  in YAML, make sure to add quotes!
- `heuristics` - Optional. Should be a list of regexes matching syntax unique to the
  language.
- `matchers` - Required. Each key in this object is optional, but *at least one* must
  be defined.
  - `extensions` - A list of file extensions that the language is associated with. E.g. `rs` for Rust.
  - `filenames` - A list of filenames that the language is associated with. E.g. `Dockerfile` for Docker.
  - `interpreters` - Executables associated with the language. Used for matching shebangs. E.g. `python3` for Python.
  - `patterns` - Advanced patterns for the full filepath. This is useful for edge cases, like the containing directory or a sub-extension changing the language.
- `priority` - Optional, defaulting to `50`. When all else fails, and we fail narrow down to a single language, matching languages are sorted by priority to pick one.

Note that regexes in this file use the syntax from the [`regex` crate][regex-syntax].

All entries in this file should be alphabetized. There is a
[script to check this file][check-languages-script] that is run
by the CI. But surely you'll run it locally so that the CI passes the first
time, right? ;)

### File Attributes

This tool also tries to detect if a file is documentation, generated, or vendored.
These can be a bit too complex to be managed with a data file, so you'll need to
write some Rust if you want to update the behavior for these.

Check `documentation.rs`, `generated.rs`, and `vendored.rs` in [`gengo/src`][lib-src]
to update detection for these. For performance reasons, checks that *don't* require
reading contents should always be done before checks that *do* read contents.

[check-languages-script]: ./scripts/check-languages-file.rb
[insta]: https://crates.io/crates/insta
[languages-file]: ./gengo/languages.yaml
[lib-src]: ./gengo/src
[regex-syntax]: https://docs.rs/regex/latest/regex/#syntax