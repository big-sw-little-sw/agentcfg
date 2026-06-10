#!/usr/bin/env sh
prek run --all-files --skip no-commit-to-branch
cargo test --workspace
