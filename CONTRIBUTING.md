# Contributing

Thanks for contributing to `agentcfg`. This is a Rust workspace that manages Agent Configuration as repeatable desired state.

## Prerequisites

- [Rust](https://rustup.rs/) (the repo pins the toolchain via `rust-toolchain.toml`; `clippy` and `rustfmt` are included)
- [prek](https://prek.j178.dev/) for Git hooks

Install prek (pick one):

```sh
brew install prek
# or: cargo install --locked prek
# or: uv tool install prek
```

## Setup

```sh
git clone <repo-url>
cd agentcfg
prek install
```

`prek install` wires Git to run hooks from [`prek.toml`](prek.toml) on each commit.

## Validate changes

Before opening a PR, run full repo validation:

```sh
scripts/validate-all.sh
```

The script runs repo hooks and then tests:

```sh
prek run --all-files --skip no-commit-to-branch
cargo test --workspace
```

The `prek` command skips only the branch-protection hook so validation can run on any local branch. Hooks include file hygiene checks plus `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings`. Tests intentionally run separately through Cargo.

## Project layout

| Crate | Role |
| --- | --- |
| `crates/agentcfg-cli` | CLI parsing, output, exit codes |
| `crates/agentcfg-core` | Config, paths, desired state, workflows, discovery |

Read before making non-trivial changes:

- [CONTEXT.md](CONTEXT.md) — canonical terminology
- [docs/prd.md](docs/prd.md) — product intent
- [docs/design-v1.md](docs/design-v1.md) — architecture and safety rules

## Conventions

- Use product terms from `CONTEXT.md` in CLI help, errors, and diagnostics.
- Include tests with behavior changes when practical.
- Do not commit directly to protected branches (`no-commit-to-branch` is enabled in prek).

## License

By contributing, you agree that your contributions are licensed under the BSD 3-Clause License ([LICENSE](LICENSE)).
