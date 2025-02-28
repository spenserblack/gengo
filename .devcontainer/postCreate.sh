#!/bin/sh
git remote add gengo https://github.com/spenserblack/gengo
git fetch gengo test/javascript:test/javascript

cargo install --locked cargo-insta taplo-cli
