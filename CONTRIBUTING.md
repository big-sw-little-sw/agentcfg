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

```sh
cargo test --workspace
```

Before opening a PR, you can run the same checks locally:

```sh
prek run --all-files
```

Hooks include file hygiene checks plus `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings`.

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
- Keep TOML field names stable (`scope`, `include`, `groups`, `skill_aliases`).
- Include tests with behavior changes when practical.
- Do not commit directly to protected branches (`no-commit-to-branch` is enabled in prek).

## License

By contributing, you agree that your contributions are licensed under the BSD 3-Clause License ([LICENSE](LICENSE)).
