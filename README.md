# gengo

Like linguist, but in Rust

## Features

- Can be run on a git revision(?)
- Nested languages(?)
- Detect by
  - extension (`.js`)
  - shebang (`#!/bin/ruby`)
  - filename (`Makefile.foo`)
  - heuristic
- Simple library
  ```rust
  let languages = Language::detect();
  let name = languages[0].name();
  let size = languages[0].size();
  let first_block_name = languages[0].children()[0].size();
  ```
