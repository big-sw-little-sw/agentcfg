# 01: Workspace, Modules, And Shared Primitives

## Goal

Replace the smoke-test workspace with the shallow Option 2B V1 module map and shared primitive types that later contracts use.

## Read First

- `docs/v1-plan/m1-stable-contracts-and-workspace-boundary/README.md`
- `CONTEXT.md`
- `docs/design-v1.md` sections: "Module Layout", "CLI Layout", "Manifest Model", and "Content Digest"

## Scope

- Remove the `greet` smoke-test API and CLI behavior.
- Create the shallow `agentcfg-core` module map around these top-level policy modules:
  - `config`
  - `resolution`
  - `installation`
  - `planning`
  - `execution`
- Add Skill-specific submodules under the policy modules where normalized item contracts or item policy need a home, such as `config::skills`, `resolution::skills`, `installation::skills`, `planning::skills`, and `execution::skills`.
- Keep command workflow modules under `workflow`.
- Keep narrow infrastructure modules for persisted files, stores, client discovery registry, content digests, and filesystem probing.
- Add one short module-level responsibility comment to each new module.
- Add core dependencies for `serde` and `thiserror`.
- Define `AgentcfgResult<T>` and an intentionally skeletal `AgentcfgError`.
- Define shared primitive enums and newtypes:
  - `Client`
  - `ClientSelection`
  - `ConfigLayerKind`
  - `InstallLevel`
  - `SourceSkillName`
  - `DiscoveryName`
  - `ConfigSourceId`
  - `ClientDiscoveryLocation`
  - `TreeDigest`

## Implementation Notes

- Do not create or depend on pre-Option 2B module names.
- Do not create deeper internal files unless this task needs their public contract.
- V1 item modules are Skill-specific. If V2 adds Subagents, add sibling submodules such as `config::subagents` and typed aggregate fields rather than a generic configured-item framework.
- `Client` is exhaustive for V1: Codex, Pi, OpenCode, Claude Code, Cline, Cursor.
- `ClientSelection` represents `AllSupported` or explicit supported Clients. Do not add `Client::All`.
- `ConfigLayerKind` and `InstallLevel` must stay separate.
- Shared string/path newtypes exist only where plain values are easy and dangerous to swap.
- Newtype fields should be private, with `new`, `as_str` or equivalent accessors, and `Display` where useful.
- Do not add validation grammar to newtype constructors unless the active docs already define it.
- Keep `AgentcfgError` empty or near-empty; add real variants only with behavior that can produce them.

## Out Of Scope

- Config parsing.
- TOML read/write.
- Filesystem probing.
- Real workflow execution.
- Trait abstractions for stores, planners, or generic configured-item frameworks unless a later task proves they are needed.

## Acceptance Criteria

- The workspace compiles without the old greeting API.
- The Option 2B V1 module names exist where later tasks expect them.
- Module comments provide local context without duplicating the design document.
- Shared primitive names make `Scope` ambiguity impossible in code.

## Validation

```bash
cargo check --workspace
```

Run `scripts/validate-all.sh` before completing M1.
