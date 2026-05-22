# agentcfg V1 Implementation Plan

This document breaks V1 into implementation milestones. Product intent lives in [prd.md](prd.md); technical contracts live in [design-v1.md](design-v1.md). If this plan conflicts with `design-v1.md`, treat `design-v1.md` as authoritative and update the plan.

Note: later milestones may change as earlier milestone implementations expose better boundaries, hidden dependencies, or simpler sequencing. Keep those changes explicit and preserve the acceptance criteria unless the product/design docs are also updated.

## Milestones

### M1: CLI and Config Skeleton

#### Task M1.1: CLI command surface

- [ ] Define CLI commands and flags.
- [ ] Stub command handlers for V1 commands.
- [ ] Add initial error reporting conventions.

#### Task M1.2: Config discovery and parsing

- [ ] Load shared project, personal project, and user config paths.
- [ ] Parse V1 config schema.
- [ ] Validate `scope` against config location.

### M2: Path Sources and Skill Discovery

#### Task M2.1: Path source discovery

- [ ] Discover skill directories containing `SKILL.md`.
- [ ] Support selected skills via `include`.
- [ ] Detect missing selected-skill validation errors.

#### Task M2.2: Source-local groups

- [ ] Parse optional source-local `skills.toml` groups.
- [ ] Resolve `groups` selections.
- [ ] Detect missing groups and group skill references.

#### Task M2.3: Installed-name resolution

- [ ] Apply aliases logically during resolution, without rewriting files yet.
- [ ] Detect installed-name collisions before aliasing and after aliasing.
- [ ] Include layer/source context in collision diagnostics.

### M3: Materialization and Hashing

#### Task M3.1: Safe source tree materialization

- [ ] Implement safe materialization for path sources.
- [ ] Materialize internal symlinks.
- [ ] Reject external symlinks, broken symlinks, and special files.

#### Task M3.2: Deterministic tree hashing

- [ ] Implement deterministic SHA-256 tree hashing.
- [ ] Normalize paths and sort entries according to the design contract.
- [ ] Return hashes with the `sha256:` prefix.

#### Task M3.3: Alias rewrite during materialization

- [ ] Apply aliases by rewriting managed `SKILL.md` frontmatter names.
- [ ] Preserve upstream source content unchanged.
- [ ] Record source and installed hashes separately.

### M4: Lockfiles and Managed Sources

#### Task M4.1: Lockfile schema and persistence

- [ ] Write adjacent lockfiles for active config layers.
- [ ] Read existing lockfiles.
- [ ] Represent source hash, installed hash, original name, installed name, and alias rewrite state.

#### Task M4.2: Plain sync locked-source behavior

- [ ] Reuse locked managed source trees for plain `sync`.
- [ ] Recreate missing managed source trees only when current source matches locked hash.
- [ ] Fail clearly when the source is unavailable or changed from the locked hash.
- [ ] Keep sync direction one-way: source to managed source tree to client target.

#### Task M4.3: Path-source upgrade behavior

- [ ] Implement `plan --upgrade` for path sources without persistent writes.
- [ ] Implement `sync --upgrade` for path sources.
- [ ] Update active lockfiles during `sync --upgrade`.
- [ ] Materialize refreshed managed source trees during `sync --upgrade`.

### M5: Planner and Sync

M5 owns the minimal manifest writes needed for safe sync updates. M6 builds on that state for stale consumer detection and pruning.

#### Task M5.1: Desired target model and client registry

- [ ] Build desired target state from active layers.
- [ ] Resolve built-in client target paths.
- [ ] Represent shared target paths across clients.

#### Task M5.2: Planner diff

- [ ] Generate plan entries for target creates.
- [ ] Generate plan entries for target updates.
- [ ] Generate plan entries for consumer additions.
- [ ] Compute stale consumer and stale artifact entries for reporting only.

#### Task M5.3: Read-only plan rendering

- [ ] Render read-only `plan` output.
- [ ] Render lockfile changes that would be created or updated.
- [ ] Verify `plan` performs no persistent writes.

#### Task M5.4: Sync apply for creates and updates

- [ ] Apply `sync` creates and updates.
- [ ] Add consumers required by desired state.
- [ ] Write the minimal manifest records needed to validate future managed target updates.
- [ ] Warn after `sync` when stale artifacts remain.

### M6: Manifest, Consumers, and Prune

#### Task M6.1: Manifest reconciliation

- [ ] Write project and user manifests.
- [ ] Read existing project and user manifests.
- [ ] Track structured consumers by `{scope, client}`.
- [ ] Preserve manifest-owned target metadata needed for safety checks.

#### Task M6.2: Shared consumers and stale state

- [ ] Merge consumers for shared target paths.
- [ ] Detect stale consumers.
- [ ] Detect stale managed artifacts.
- [ ] Cover shared `.agents/skills` consumers across Codex/Pi/OpenCode/Cursor.

#### Task M6.3: Prune safety engine

- [ ] Implement `prune` and `prune --user`.
- [ ] Remove stale consumers.
- [ ] Remove target artifacts only when no consumers remain.
- [ ] Refuse to prune unexpected symlink targets or unmanaged files.
- [ ] Delete directories only if empty and manifest-owned.

### M7: Status and Doctor

#### Task M7.1: Status

- [ ] Implement `status` and `status --user`.
- [ ] Report installed managed artifacts.
- [ ] Report broken symlinks, unexpected targets, missing managed sources, stale artifacts, and config/lock mismatch.
- [ ] Report unmanaged artifacts in configured target directories as informational unless they conflict.

#### Task M7.2: Doctor

- [ ] Implement `doctor` environment checks.
- [ ] Check git availability, repo root detection, supported clients, path writability, and config schema validity.
- [ ] Report target confidence and client support diagnostics.
- [ ] Include optional network/source checks where appropriate.

### M8: Git Sources

#### Task M8.1: Git source resolution

- [ ] Resolve git sources.
- [ ] Resolve floating refs.
- [ ] Record requested refs and resolved commits in lockfiles.

#### Task M8.2: Git materialization through the source pipeline

- [ ] Discover skills from resolved git source content.
- [ ] Materialize git source skills into managed source trees.
- [ ] Reuse existing safe materialization and hashing behavior.

#### Task M8.3: Plain sync from locked git state

- [ ] Reuse locked managed git source trees for plain `sync`.
- [ ] Recreate missing managed git source trees from locked commits when available.
- [ ] Fail clearly when a locked commit cannot be fetched or restored.

#### Task M8.4: Git upgrade behavior

- [ ] Implement git behavior for `plan --upgrade`.
- [ ] Implement git behavior for `sync --upgrade`.
- [ ] Update lockfiles only during `sync --upgrade`.

## Validation

- [ ] Automated tests pass.
- [ ] `agentcfg init` creates personal project config.
- [ ] `agentcfg init --project` creates shared project config.
- [ ] `agentcfg init --user` creates user config.
- [ ] `agentcfg plan` performs no persistent writes.
- [ ] `agentcfg sync` installs a path-source skill from locked managed state.
- [ ] `agentcfg sync --upgrade` imports changed path-source content.
- [ ] `agentcfg prune` removes only manifest-owned stale artifacts.
- [ ] Alias collision behavior is covered.
- [ ] Internal symlink materialization and external symlink rejection are covered.
- [ ] Shared `.agents/skills` consumers across Codex/Pi/OpenCode/Cursor are covered.

## Open Planning Questions

- [X] Should git sources be included before or after the first path-source sync milestone? YES. we should
- [ ] How much source provenance should be exposed in normal command output versus `doctor`?
- [ ] Should V1 support both `skills/<name>/SKILL.md` and root-level `<name>/SKILL.md` source layouts?
