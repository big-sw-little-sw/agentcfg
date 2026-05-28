# Agent instructions

`agentcfg` manages Agent Configuration as repeatable desired state. This file is stable policy for coding agents. **Current implementation status** is in [README.md](README.md) § Status.

## Read first

1. [CONTEXT.md](CONTEXT.md) — canonical terms for user-facing strings and new APIs.
2. [docs/design-v1.md](docs/design-v1.md) — persisted contracts and safety rules.
3. [docs/implementation-plan-v1.md](docs/implementation-plan-v1.md) — milestones and task boundaries for the work you are doing.
4. [docs/prd.md](docs/prd.md) — product intent and command behavior.

If `design-v1.md` and `implementation-plan-v1.md` disagree, **stop and ask** which source to follow for that change. Do not guess.

## Repository layout

| Crate | Role |
| --- | --- |
| `crates/agentcfg-cli` | Argument parsing, terminal output, exit codes. Adapters only — no domain orchestration. |
| `crates/agentcfg-core` | Config, paths, Skill Sources, desired state, lockfiles, manifests, workflows, discovery registry. |

Use the toolchain pinned in [rust-toolchain.toml](rust-toolchain.toml). Validate with:

```sh
cargo test --workspace
```

## Concepts → code

| Term | Module(s) |
| --- | --- |
| Config Layer, Persisted Scope Value | `layer_level`, `config`, `config_paths` |
| Install Level | `layer_level`, `workflow` request types, CLI `--user` |
| Skill Source, Skill Selection, Skill Alias (config shape) | `config`; resolution in `skill_source` |
| Client Discovery Location, Client Discovery Registry | `discovery_registry`, init scan in `workflow::init` |
| Managed State, lockfile, manifest paths | `config_paths` (`ManagedStatePaths`, `ConfigFilePaths`) |
| Desired State, Locked Desired State, Configured Item | `desired_state` |
| Lockfile | `lockfile` |
| Manifest, Discovery Requirement, Installed Artifact | `manifest` |
| Preview / Apply / Source Refresh | `workflow` (orchestration); CLI `preview` / `apply` |
| Status / Prune / Doctor install-health terms | `install_health` (placeholder); behavior in `workflow` when implemented |

Stable TOML field names (`scope`, `include`, `groups`, `skill_aliases`) are intentional; do not rename them to match glossary prose.

## Imports

- Use `agentcfg_core::layer_level::{ConfigLayer, InstallLevel}` for Config Layer and Install Level selectors.
- Use `agentcfg_core::workflow` for workflow request/result types and `init` / `preview` / `apply` entrypoints.

## Conventions

- Match ubiquitous language in CLI help, errors, and diagnostics.
- Keep changes scoped to the task; add tests with behavior in the same change when possible.
- Prefer focused core modules over growing `workflow` with low-level logic.
- Scope and allowed behavior come from the task plus `design-v1.md` and `implementation-plan-v1.md` — not from this file.
