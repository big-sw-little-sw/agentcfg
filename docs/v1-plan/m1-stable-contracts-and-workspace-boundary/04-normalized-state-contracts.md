# 04: Normalized Config And Installation Contracts

## Goal

Define normalized in-memory contracts passed between config assembly, resolution, installation observation, planning, execution, and reporting seams.

## Read First

- `docs/v1-plan/m1-stable-contracts-and-workspace-boundary/README.md`
- `docs/design-v1.md` sections: "Data Shapes", "Command Composition Types", and "Manifest Model"
- Task 01 output for shared primitives
- Task 03 output for persisted schema names that normalized contracts should not leak

## Scope

- Define aggregate contract shells:
  - `ConfigRequest`
  - `PinnedConfig`
  - `LockfilePinnedConfig`
  - `PlannedPinnedConfig`
  - `ObservedInstallation`
- Define Skill-specific shells under those aggregates.
- Define each aggregate with typed Skill fields rather than a generic resource map:
  - `ConfigRequest { skills: skills::SkillConfigRequest }`
  - `PinnedConfig { skills: skills::PinnedSkillConfig }`
  - `LockfilePinnedConfig(PinnedConfig)`
  - `PlannedPinnedConfig(PinnedConfig)`
  - `ObservedInstallation { skills: skills::ObservedSkillInstallation }`
- Define resolution, planning, report, and execution containers:
  - `Lockfiles`
  - `ResolutionPlan`
  - `LockfileConfigCheck`
  - `LockfileChange`
  - `ResolutionDiagnostic`
  - `BlockingConfigRequestDiagnostic`
  - `PreviewReport`
  - `ApplyPlan`
  - `PrunePlan`
  - `StatusReport`
  - `ApplyResult`
  - `PruneResult`
- Define normalized artifact and requirement identity structs where they are used across seams.

## Implementation Notes

- Planning receives normalized pinned config and observed installation, not lockfile schema or filesystem snapshots.
- Normalized contracts should use canonical domain names, not persisted TOML field names.
- Do not derive serde for internal config, resolution, installation, planning, or execution structs unless they cross a persisted boundary.
- Use comments to mark fields intentionally left skeletal for later policy work.
- Keep lifecycle policy out of the contract structs.
- V1 remains Skill-specific. If V2 adds Subagents, add sibling submodules and typed fields such as `subagents: subagents::SubagentConfigRequest`; do not introduce a generic resource framework in M1.

## Out Of Scope

- Conversion from persisted schema to `ConfigRequest`.
- Source resolution.
- Observed-installation filesystem reads.
- Planning classification.
- Reporter formatting.

## Acceptance Criteria

- Module seams can exchange normalized contracts without referring to TOML schemas.
- Config request, pinned config, lockfile-pinned config, planned pinned config, and observed installation are distinct types.
- Lockfile-pinned and planned-pinned wrappers share `PinnedConfig` rather than duplicating separate resource shapes.
- Artifact identity and Discovery Requirement identity are explicit enough to support the Manifest safety model later.
- No generic resource map, bag, or trait is introduced for V1 Skill contracts.

## Validation

```bash
cargo check --workspace
```

Run `scripts/validate-all.sh` before completing M1.
