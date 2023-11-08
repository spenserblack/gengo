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
| **Analyze Directory** | No | Yes | Yes |
| **Requires Git Repository** | Yes | No | No |
| **Detect Language by Extension** | Yes | Yes | Yes |
| **Detect Language by Filename** | Yes | Yes | Yes |
| **Detect by Filepath Pattern** | No | No | Yes |
| **Detect Language with Heuristics** | Yes | No | Yes |
| **Detect Language with Classifier** | Yes | No | Not Yet ;) |

## Installation

View [the installation documentation][install-docs].

## Usage

This tool has multiple file sources. Each file source can have unique usage to take advantage of its
strengths and work around its weaknesses.

### Directory File Source


This is a very generic file source that tries not to make many assumptions about your environment
and workspace.

#### Ignoring Files

You can utilize a `.gitignore` file and/or an `.ignore` file to prevent files from
being scanned. See the [`ignore`][ignore-crate] for more details.

### Git File Source

The git file source is highly opinionated -- it tries to act like a git utility, and uses git tools.
Its goal is to behave similarly to [linguist].

#### Overrides

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

[ignore-crate]: https://docs.rs/ignore
[install-docs]: ./docs/INSTALLATION.md
[linguist]: https://github.com/github-linguist/linguist
[tokei]: https://github.com/xampprocky/tokei
