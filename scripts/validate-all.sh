#!/usr/bin/env sh
set -eu

prek run --all-files --skip no-commit-to-branch
cargo test --workspace
