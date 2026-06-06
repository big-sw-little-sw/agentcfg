# M1: Stable Contracts And Workspace Boundary

M1 establishes the Rust workspace contracts that later milestones fill in. It freezes seam-level APIs and normalized contract shapes, not install behavior.

Design note:

- Design risk: Tier 2.
- Chosen shape: full shallow Option 2B module map, typed seam signatures, persisted schema structs, normalized config/installation contracts, and a CLI command surface that parses but does not execute workflows.
- Why not simpler: data structs alone would not protect module boundaries or the CLI/core split.
- Why not more abstract: V1 manages Skills only; M1 should preserve a V2 Subagent path through sibling item modules and typed fields, not a generic resource graph or trait-heavy platform layer.
- Next plausible change considered: M2 and M3 can replace stubs with leaf behavior and policy without renaming the public seams.

## Task Plans

Each task file is meant to be executable by an agent after reading this README, `CONTEXT.md`, `docs/prd.md`, and `docs/design-v1.md`. The task files avoid copying the full design, but they name the design sections and decisions they depend on.

1. [Workspace, Modules, And Shared Primitives](01-workspace-modules-and-shared-primitives.md)
2. [Command And Boundary Contracts](02-command-and-boundary-contracts.md)
3. [Persisted File Schemas](03-persisted-file-schemas.md)
4. [Normalized Config And Installation Contracts](04-normalized-state-contracts.md)
5. [CLI Surface And Validation](05-cli-surface-and-validation.md)

## M1 Decisions

- Public seam signatures return typed `AgentcfgResult<_>`.
- Stub bodies may use `unimplemented!("... is not implemented yet")` while they are not reachable from CLI execution.
- New modules get a short module-level responsibility comment.
- Comments on types/functions are added only when names and shapes do not convey intent, invariants, constraints, or trade-offs clearly.
- Persisted TOML uses kebab-case field names.
- Config files use `config-layer`, not `scope`.
- Skill Source config lives under `[[skills.sources]]`.
- Per-source alias rules use `aliases`, not `skill-aliases`.
- `Client` is an exhaustive enum of V1 supported Clients.
- `ClientSelection` is `AllSupported` or explicit supported Clients; `all` is not a `Client`.
- `ConfigLayerKind` and `InstallLevel` are separate types.
- Minimal shared newtypes: `SourceSkillName`, `DiscoveryName`, `ConfigSourceId`, `ClientDiscoveryLocation`, and `TreeDigest`.
- Lockfile schema is skeletal.
- Normalized cross-module contracts are `ConfigRequest`, `PinnedConfig`, `LockfilePinnedConfig`, `PlannedPinnedConfig`, and `ObservedInstallation`.
- Resolution contracts use `Lockfiles`, `ResolutionPlan`, `LockfileConfigCheck`, and `ResolutionDiagnostic`.
- Apply and prune execution results are `ApplyResult` and `PruneResult`.
- V1 uses Skill-specific submodules and typed fields; V2 Subagents should be added as sibling submodules and fields, not through a generic resource framework.
- Manifest schema is close to final and uses list records rather than encoded TOML map keys.
- Do not add disposable scaffold tests that will lose meaning once V1 behavior exists.

## Validation Baseline

Use the repo baseline after code changes:

```bash
scripts/validate-all.sh
```

Focused `cargo check --workspace` or `cargo test --workspace` is fine while iterating, but final M1 validation should use the repo script when feasible.
