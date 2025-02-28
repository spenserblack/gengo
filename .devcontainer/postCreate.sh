#!/bin/sh
cargo install --locked cargo-insta taplo-cli

git remote add gengo https://github.com/spenserblack/gengo
git fetch gengo test/javascript:test/javascript
