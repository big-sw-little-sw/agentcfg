# Agent instructions

`agentcfg` manages Agent Configuration as repeatable pinned configuration. This file is stable policy for coding agents.

## Read first

1. [CONTEXT.md](CONTEXT.md) — canonical terms for user-facing strings and new APIs.
2. [docs/prd.md](docs/prd.md) — product intent and command behavior.
3. [docs/design-v1.md](docs/design-v1.md) — V1 architecture, contracts, and safety rules.

Do **not** read files under `docs/archive/` unless the user specifically asks you to inspect archived material. Archived files are historical context, not active design or implementation guidance.

## Implementation status

The repository currently has a minimal Rust Cargo workspace for `agentcfg-core` and `agentcfg-cli`. V1 implementation is still early; use the product terms and safety rules from `CONTEXT.md`, `docs/prd.md`, and `docs/design-v1.md` when adding real behavior.

The V1 Cargo workspace boundary described in `docs/design-v1.md` remains the active architectural target unless a newer approved plan changes it.

## Validation

Run full repo validation with:

```bash
scripts/validate-all.sh
```

The script runs:

```bash
prek run --all-files --skip no-commit-to-branch
cargo test --workspace
```

The `prek` command disables only the branch-protection hook so agents can validate on any working branch. Do not skip other hooks unless a task-specific reason requires it, and report any validation command that was not run.

`prek` owns file hygiene plus Rust formatting and linting. Tests intentionally run separately through Cargo. Focused commands such as targeted package tests are fine during iteration, but final validation for code changes should use `scripts/validate-all.sh` when feasible.

## Conventions

- Match ubiquitous language in CLI help, errors, and diagnostics.
- Keep changes limited to the task; add tests with behavior in the same change when implementation code exists.
- Prefer focused core modules over growing workflow orchestration with low-level logic.
- For new modules, add a short module-level comment describing its responsibility. Add comments on types/functions only when the name and shape cannot convey intent, invariants, constraints, or trade-offs clearly.
- Scope and allowed behavior come from the task plus `CONTEXT.md`, `docs/prd.md`, and `docs/design-v1.md` — not from this file.
