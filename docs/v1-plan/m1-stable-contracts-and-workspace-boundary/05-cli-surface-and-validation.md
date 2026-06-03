# 05: CLI Surface And Validation

## Goal

Expose the V1 command surface through `clap` while keeping workflow execution unwired until later milestones.

## Read First

- `docs/v1-plan/m1-stable-contracts-and-workspace-boundary/README.md`
- `docs/prd.md` sections: "Commands" and "Status and Doctor"
- `docs/design-v1.md` sections: "CLI Layout", "Preview And Reporting Terms", and "Command Flows"
- Task 02 output for command request types

## Scope

- Add CLI dependencies for `clap` and `anyhow`.
- Add `args.rs`, `exit_codes.rs`, and `output.rs` modules.
- Define the V1 command surface:
  - `agentcfg init [--project | --user]`
  - `agentcfg preview [--user] [--refresh-sources] [--client <client>...]`
  - `agentcfg apply [--user] [--refresh-sources] [--client <client>...]`
  - `agentcfg prune [--user] [--client <client>...]`
  - `agentcfg status [--user] [--client <client>...]`
  - `agentcfg doctor`
- Map CLI args into request-shaped values where that does not require workflow execution.
- Make real command execution return a CLI-level "not implemented yet" diagnostic and exit `1`.
- Do not call core workflow stubs from CLI execution in M1.

## Implementation Notes

- `--user` means User Config only for `init`; it means User Level for `preview`, `apply`, `prune`, and `status`.
- `--refresh-sources` is accepted only for `preview` and `apply`.
- `--client` may repeat for `preview`, `apply`, `prune`, and `status`.
- `doctor` has no `--user`.
- Exit-code mapping belongs in the CLI crate.
- Core errors should not be flattened in a way that loses future exit-code mapping.

## Out Of Scope

- Workflow execution.
- Terminal report rendering.
- Snapshot tests for placeholder output.
- Config file creation from `init`.

## Acceptance Criteria

- `agentcfg --help` and subcommand help show the V1 surface.
- Non-help command execution fails cleanly with exit `1` instead of panicking.
- CLI parsing does not require filesystem state.
- No disposable scaffold tests are added unless a CLI help contract is expected to survive V1.

## Validation

```bash
cargo check --workspace
scripts/validate-all.sh
```
