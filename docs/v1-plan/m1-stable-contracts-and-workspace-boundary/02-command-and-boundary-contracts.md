# 02: Command And Boundary Contracts

## Goal

Define command request/result vocabulary and public seam function signatures without implementing behavior.

## Read First

- `docs/v1-plan/m1-stable-contracts-and-workspace-boundary/README.md`
- `docs/design-v1.md` sections: "CLI Layout", "Command Composition Types", "Module Responsibilities", and "Command Flows"
- Task 01 output for shared primitives and `AgentcfgResult`

## Scope

- Add command request types:
  - `InitRequest`
  - `PreviewRequest`
  - `ApplyRequest`
  - `PruneRequest`
  - `StatusRequest`
  - `DoctorRequest`
- Add skeletal command output/result types:
  - `PreviewCommandPlan`
  - `ApplyCommandPlan`
  - `ApplyCommandResult`
  - `PruneCommandPlan`
  - `PruneCommandResult`
  - `CommandExecutionOutcome`
  - `StatusCommandReport`
  - `DoctorReport`
- Add public seam signatures for:
  - `workflow::{init, preview, apply, prune, status, doctor}`
  - `config`
  - `resolution`
  - `installation`
  - `planning`
  - `execution`
  - store and filesystem probe modules

## Implementation Notes

- Public seam functions return `AgentcfgResult<_>`.
- Stub bodies may use `unimplemented!("... is not implemented yet")` while they are not reachable from CLI execution.
- Do not include milestone numbers in transient stub messages.
- `doctor` stays structurally separate from `status`; it does not use resolution, installation planning, or execution planning.
- Reporter/output types should be skeletal containers with terse comments for later rendering work.

## Out Of Scope

- Real source resolution.
- Observed-installation collection.
- Planning classification logic.
- Execution preflight or mutation.
- Terminal output wording.

## Acceptance Criteria

- Later tasks can call the intended seams by name and type.
- CLI wording and filesystem details do not leak into config, resolution, planning, or execution contracts.
- Stubbed workflow seams are not wired into normal CLI command execution.

## Validation

```bash
cargo check --workspace
```

Run `scripts/validate-all.sh` before completing M1.
