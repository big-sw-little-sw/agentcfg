# Agent instructions

`agentcfg` manages Agent Configuration as repeatable pinned configuration. This file is stable policy for coding agents.

## Read first

1. [docs/UBIQUITOUS_LANGUAGE.md](CONTEXT.md) — canonical terms for user-facing strings and new APIs.

Do **not** read files under `docs/archive/` unless the user specifically asks you to inspect archived material. Archived files are historical context, not active design or implementation guidance.

## Implementation status

The repository currently has a minimal Rust Cargo workspace for `agentcfg-core` and `agentcfg-cli`.

## Validation

Run full repo validation with:

```bash
scripts/validate-all.sh
```

## Conventions

- Match ubiquitous language in CLI help, errors, and diagnostics.
- Keep changes limited to the task; add tests with behavior in the same change when implementation code exists.
- Prefer focused core modules over growing workflow orchestration with low-level logic.
- For new modules, add a short module-level comment describing its responsibility. Add comments on types/functions only when the name and shape cannot convey intent, invariants, constraints, or trade-offs clearly.
