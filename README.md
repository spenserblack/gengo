# gengo (言語)

[![library](https://img.shields.io/crates/v/gengo.svg?label=gengo)](https://crates.io/crates/gengo)
[![binary](https://img.shields.io/crates/v/gengo-bin.svg?label=gengo-bin)](https://crates.io/crates/gengo-bin)
[![CI](https://github.com/spenserblack/gengo/actions/workflows/ci.yml/badge.svg)](https://github.com/spenserblack/gengo/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/spenserblack/gengo/branch/main/graph/badge.svg?token=ihIEUQWwSt)](https://codecov.io/gh/spenserblack/gengo)

A [linguist][linguist]-inspired language classifier with multiple file source handlers

## Comparison

|          Feature/Behavior           | [linguist][linguist] |   gengo    |
| :---------------------------------: | :------------------: | :--------: |
|      **Analyze Git Revision**       |         Yes          |    Yes     |
|        **Analyze Directory**        |          No          |    Yes     |
|     **Requires Git Repository**     |         Yes          |     No     |
|  **Detect Language by Extension**   |         Yes          |    Yes     |
|   **Detect Language by Filename**   |         Yes          |    Yes     |
|   **Detect by Filepath Pattern**    |          No          |    Yes     |
| **Detect Language with Heuristics** |         Yes          |    Yes     |
| **Detect Language with Classifier** |         Yes          | Not Yet ;) |

## Installation

[![Packaging status](https://repology.org/badge/vertical-allrepos/rust%3Agengo.svg)](https://repology.org/project/rust%3Agengo/versions)

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
Basically, just replace `linguist-FOO` with `gengo-FOO`. _Unlike_ linguist,
`gengo-detectable` will _always_ make a file be included in statistics (linguist
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
# Use the Language enum's variant name (see docs.rs for more details)
templates/*.js gengo-language=PlainText
```

You will need to commit your `.gitattributes` file for it to take effect.

[ignore-crate]: https://docs.rs/ignore
[install-docs]: ./docs/INSTALLATION.md
[linguist]: https://github.com/github-linguist/linguist
