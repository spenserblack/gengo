# Gengo

## Installating the binary

```shell
cargo install gengo-bin
```

## Usage

[API documentation][docs-rs]

### Overrides

Like [linguist][linguist], you can override behavior using a `.gitattributes` file.
Basically, just replace `linguist-FOO` with `gengo-FOO`. *Unlike* linguist,
`gengo-detectable` will *always* make a file be included in statistics (linguist
will still exclude them if they're generated or vendored).

```gitattributes
# .gitattributes

# boolean attributes:

# These can be *negated* by prefixing with `-` (`-gengo-documentation`).
# Mark a file as documentation
*.html gengo-documentation
# Mark a file as generated
my-built-files/* gengo-generated
# Mark a file as vendored
deps/* gengo-vendored

# string attributes:
# Override the detected language for a file
# Spaces must be replaced with hyphens. Case-insensitive.
templates/*.js gengo-language=Plain-Text
```

You will need to commit your `.gitattributes` file for it to take effect.

[docs-rs]: https://docs.rs/gengo
[linguist]: https://github.com/github-linguist/linguist
