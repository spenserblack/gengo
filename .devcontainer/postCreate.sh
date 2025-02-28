#!/bin/sh
cargo install --locked cargo-insta taplo-cli

# Get remote URL
origin=$(git config --get remote.origin.url)

if [ "$origin" = "https://github.com/spenserblack/gengo" ]; then
  git fetch origin test/javascript:test/javascript
else
  git fetch upstream test/javascript:test/javascript
fi
