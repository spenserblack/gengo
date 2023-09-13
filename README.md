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

## Installation

### Install Script

Behind the scenes, this script downloads a binary from the latest GitHub release.

#### Unix

```shell
curl https://raw.githubusercontent.com/spenserblack/gengo/main/install.sh | sh
```

### From GitHub Releases

Download the appropriate file from [a release](https://github.com/spenserblack/gengo/releases).

### With `cargo`

The following installs the `gengo` binary.

```shell
cargo install gengo-bin
```

### Docker

You can build a docker image and run it as well.

```bash
docker build -t gengo .
docker run --rm -v $(pwd):$(pwd) -w $(pwd) -t gengo
```

Or pull it from the [GitHub Container Registry][ghcr-package].

## Usage

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

[ghcr-package]: https://github.com/users/spenserblack/packages/container/package/gengo
[linguist]: https://github.com/github-linguist/linguist
[tokei]: https://github.com/xampprocky/tokei
