# gengo (言語)

[![library](https://img.shields.io/crates/v/gengo.svg?label=gengo)](https://crates.io/crates/gengo)
[![binary](https://img.shields.io/crates/v/gengo-bin.svg?label=gengo-bin)](https://crates.io/crates/gengo-bin)
[![CI](https://github.com/spenserblack/gengo/actions/workflows/ci.yml/badge.svg)](https://github.com/spenserblack/gengo/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/spenserblack/gengo/branch/main/graph/badge.svg?token=ihIEUQWwSt)](https://codecov.io/gh/spenserblack/gengo)

A bit like [tokei][tokei], a lot like [linguist][linguist].

## Comparison

| Feature/Behavior | [linguist][linguist] | [tokei][tokei] | gengo |
| :--------------: | :------------------: | :------------: | :---: |
| **Analyze Git Revision** | Yes | No | Yes |
| **Analyze Directory** | No | Yes | No |
| **Requires Git Repository** | Yes | No | Yes |
| **Detect Language by Extension** | Yes | Yes | Yes |
| **Detect Language by Filename** | Yes | Yes | Yes |
| **Detect by Filepath Pattern** | No | No | Yes |
| **Detect Language with Heuristics** | Yes | No | Yes |
| **Detect Language with Classifier** | Yes | No | Not Yet ;) |

## Usage

### Overrides

Like [linguist][linguist], you can override behavior using a `.gitattributes` file.
Basically, just replace `linguist-FOO` with `gengo-FOO`. *Unlike* linguist,
`gengo-detectable` will *always* make a file be included in statistics (linguist
will still exclude them if they're generated or vendored).

```gitattributes
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

[linguist]: https://github.com/github-linguist/linguist
[tokei]: https://github.com/xampprocky/tokei
