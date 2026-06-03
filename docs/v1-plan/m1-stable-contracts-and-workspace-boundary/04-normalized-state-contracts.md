# 04: Normalized State Contracts

## Goal

Define normalized in-memory state aggregates passed between planning, inventory, reconciliation, execution, and reporting seams.

## Read First

- `docs/v1-plan/m1-stable-contracts-and-workspace-boundary/README.md`
- `docs/design-v1.md` sections: "State Shapes", "Desired And Locked State", "Command Composition Types", and "Manifest Model"
- Task 01 output for shared primitives
- Task 03 output for persisted schema names that normalized contracts should not leak

## Scope

- Define aggregate state shells:
  - `DesiredState`
  - `LockedDesiredState`
  - `ProposedLockedDesiredState`
  - `CurrentState`
- Define skill resource shells under those aggregates.
- Define planning/report containers:
  - `LockPlan`
  - `ExistingLockState`
  - `LockfileChange`
  - `LockDiagnostic`
  - `FatalDesiredStateDiagnostic`
  - `PreviewReport`
  - `ApplyPlan`
  - `PrunePlan`
  - `StatusReport`
  - `ApplyExecutionResult`
  - `PruneExecutionResult`
- Define normalized artifact and requirement identity structs where they are used across seams.

## Implementation Notes

- The reconciler receives normalized locked install state, not lockfile schema.
- Normalized contracts should use canonical domain names, not persisted TOML field names.
- Do not derive serde for internal planning/current/reconciler structs unless they cross a persisted boundary.
- Use comments to mark fields intentionally left skeletal for later policy work.
- Keep lifecycle policy out of the state structs.

## Out Of Scope

- Conversion from persisted schema to normalized state.
- Source resolution.
- Current-state filesystem reads.
- Reconciler classification.
- Reporter formatting.

## Acceptance Criteria

- Module seams can exchange normalized state without referring to TOML schemas.
- Desired, locked, proposed locked, and current state are distinct types.
- Artifact identity and Discovery Requirement identity are explicit enough to support the Manifest safety model later.

## Validation

```bash
cargo check --workspace
```

Run `scripts/validate-all.sh` before completing M1.
