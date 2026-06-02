# Agent instructions

`agentcfg` manages Agent Configuration as repeatable desired state. This file is stable policy for coding agents. **Current implementation status** is in [README.md](README.md) § Status.

## Read first

1. [CONTEXT.md](CONTEXT.md) — canonical terms for user-facing strings and new APIs.
2. [docs/prd.md](docs/prd.md) — product intent and command behavior.
3. [docs/design-v1.md](docs/design-v1.md) — V1 architecture, contracts, and safety rules.

Do **not** read files under `docs/archive/` unless the user specifically asks you to inspect archived material. Archived files are historical context, not active design or implementation guidance.

## Implementation status

The repository is currently a clean slate: no package workspace, source tree, tests, lockfile, or toolchain pin is present. No build or test command is defined until a new implementation is added.

When starting a new implementation, use the product terms and safety rules from `CONTEXT.md`, `docs/prd.md`, and `docs/design-v1.md`. The V1 Cargo workspace boundary described in `docs/design-v1.md` remains the active architectural target unless a newer approved plan changes it.

Stable TOML field names (`scope`, `include`, `groups`, `skill_aliases`) are intentional; do not rename them to match glossary prose.

## Conventions

- Match ubiquitous language in CLI help, errors, and diagnostics.
- Keep changes scoped to the task; add tests with behavior in the same change when implementation code exists.
- Prefer focused core modules over growing workflow orchestration with low-level logic.
- Scope and allowed behavior come from the task plus `CONTEXT.md`, `docs/prd.md`, and `docs/design-v1.md` — not from this file.
