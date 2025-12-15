# Hacking

## Getting Started

### Tools

#### EditorConfig

This project uses [EditorConfig][editorconfig]. If necessary, install the plugin
for your preferred editor(s).

#### Testing

This project uses [insta][insta] for snapshot testing. Review the instructions
for using insta and installing the `cargo insta` executable.

## Development

### Language Support

While the library does support providing your own language definitions, the
"out-of-the-box" language definitions are defined in
[`./gengo/languages.yaml`][languages-file]. This is where you register
new languages, add extensions, filenames, filepath patterns, or interpreters
to help identify languages, etc. Pretty much anything involving language
detection or the returned language data comes from this file (but there are
a few exceptions).

#### The `languages.yaml` file

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
      - "**/special-path/*.ext"
  priority: 100
```

- `category` - Required. It should be `data`, `markup`, `programming`, `prose`, or `query`.
- `color` - Required. It should be a hexadecimal color. Since `#` is a comment
  in YAML, make sure to add quotes!
- `heuristics` - Optional. Should be a list of regexes matching syntax unique to the
  language.
- `matchers` - Required. Each key in this object is optional, but *at least one* must
  be defined.
  - `extensions` - A list of file extensions that the language is associated with. E.g. `rs` for Rust.
  - `filenames` - A list of filenames that the language is associated with. E.g. `Dockerfile` for Docker.
  - `interpreters` - Executables associated with the language. Used for matching shebangs. E.g. `python3` for Python.
  - `patterns` - Glob patterns for the full filepath. This is useful for edge cases, like the containing directory or a sub-extension changing the language.
- `priority` - Optional, defaulting to `50`. When all else fails, and we fail narrow down to a single language, matching languages are sorted by priority to pick one.

Note that regexes in this file use the syntax from the [`regex` crate][regex-syntax].

All entries in this file should be alphabetized. There is a
[script to check this file][check-languages-script] that is run
by the CI. But surely you'll run it locally so that the CI passes the first
time, right? ;)

#### File Attributes

This tool also tries to detect if a file is documentation, generated, or vendored.
These can be a bit too complex to be managed with a data file, so you'll need to
write some Rust if you want to update the behavior for these.

Check `documentation.rs`, `generated.rs`, and `vendored.rs` in [`gengo/src`][lib-src]
to update detection for these. For performance reasons, checks that *don't* require
reading contents should always be done before checks that *do* read contents.

### Adding a sample

This project can test itself on code samples for accuracy. Not only that, but the samples can be used
in the future to train additional classification methods. If you would like to add a sample, add a
file to `samples-test/samples/LANGUAGE/`, where `LANGUAGE` is the name of the language as seen in
[`languages.yaml`][languages-file].

When adding a sample:
- It should preferably be a real-world sample, not a "hello world" example.
- State where the sample comes from. Did you write it? Did you copy it from an open-source repository?
  - If you wrote it yourself, you agree that the sample is licensed under this repository's licenses.
  - If you copied it from somewhere else, make sure that the code you copied is licensed under the
    MIT or Apache 2.0 license.

## Testing

Because this project analyzes git revisions, some of the tests are run on
commits in this project that are unreachable from `main`. Make sure to
fetch branches that are prefixed with `test/` from this repository.

## Updating dependencies

For dependencies that use *ranges* for their version requirement, please don't jump up several
breaking versions in a single commit. Instead, increment the breaking version by one for each
commit. This makes it easier to bisect any issues with the dependency and ensure that each
breaking version in the range has been tested. For example, if you're bumping a version requirement
from `>= 1, < 2` to `>= 1, < 5`, then you should make *3 commits* (bump to `< 3`, bump to `< 4`, and
bump to `< 5`).


[check-languages-script]: ./scripts/check-languages-file.rb
[editorconfig]: https://editorconfig.org/
[insta]: https://crates.io/crates/insta
[languages-file]: ./gengo/languages.yaml
[lib-src]: ./gengo/src
[regex-syntax]: https://docs.rs/regex/latest/regex/#syntax
