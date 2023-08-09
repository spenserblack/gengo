# gengo (言語)

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

[linguist]: https://github.com/github-linguist/linguist
[tokei]: https://github.com/xampprocky/tokei
