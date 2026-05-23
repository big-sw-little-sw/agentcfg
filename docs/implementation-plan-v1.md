# agentcfg V1 Implementation Plan

This document breaks V1 into implementation milestones. Product intent lives in [prd.md](prd.md); technical contracts live in [design-v1.md](design-v1.md). If this plan conflicts with `design-v1.md`, treat `design-v1.md` as authoritative and update the plan.

The plan is optimized for agent execution. Each task should be small enough that an implementation agent has at least 90% confidence it can complete the task from this document, the PRD, and the design doc without needing another design discussion.

## Agent Task Rules

- Keep each task focused on one ownership boundary.
- Prefer tasks that produce a compiling, testable state.
- Do not mix domain logic, persistence, filesystem mutation, and CLI rendering in one task unless the task is explicitly an end-to-end slice.
- Add tests in the same task as the behavior unless the task is scaffolding only.
- If a task exposes a hidden design decision, stop and update `design-v1.md` before implementing.
- Keep the core crate skill-first. Do not introduce generic resource abstractions before a second resource kind exists.

## Workspace Shape

V1 uses one Cargo workspace:

```text
Cargo.toml
crates/
  agentcfg-cli/
  agentcfg-core/
```

Responsibilities:

- `agentcfg-cli`: argument parsing, terminal rendering, exit codes, and command-specific interaction.
- `agentcfg-core`: config, source discovery, materialization, hashing, lockfiles, manifests, planning, apply operations, status, doctor, and client registry.

The binary name remains `agentcfg`.

## Milestones

### M0: Workspace and Test Skeleton

Goal: create a compiling workspace with clear CLI/core ownership before feature work starts.

#### Task M0.1: Create Cargo workspace

- [x] Add root `Cargo.toml` with `agentcfg-cli` and `agentcfg-core` members.
- [x] Add `crates/agentcfg-core/Cargo.toml` and `src/lib.rs`.
- [x] Add `crates/agentcfg-cli/Cargo.toml` and `src/main.rs`.
- [x] Configure the CLI package to publish a binary named `agentcfg`.
- [x] Add minimal smoke tests or compile checks for both crates.

Validation:

```sh
cargo test --workspace
cargo run -p agentcfg-cli -- --help
```

#### Task M0.2: Establish shared result and error conventions

- [ ] Add a core `Result<T>` alias and error type.
- [ ] Add CLI error-to-exit-code mapping.
- [ ] Keep terminal formatting out of core errors except concise diagnostic strings.
- [ ] Add tests for at least one CLI error mapping.

Validation:

```sh
cargo test --workspace
```

### M1: CLI Surface and Config Paths

Goal: make every V1 command invocable while keeping behavior stubbed until core operations exist.

#### Task M1.1: Define CLI command surface

- [ ] Add `init`, `plan`, `sync`, `prune`, `status`, and `doctor`.
- [ ] Add `--project`, `--user`, and `--upgrade` only where allowed by the PRD.
- [ ] Reject invalid flag combinations through argument parsing where possible.
- [ ] Route each command to a small CLI handler that calls a core stub.
- [ ] Add CLI snapshot or assertion tests for supported and rejected command forms.

Validation:

```sh
cargo test --workspace
cargo run -p agentcfg-cli -- plan --help
```

#### Task M1.2: Model config scopes and paths in core

- [ ] Add scope values: `project`, `user-project`, and `user`.
- [ ] Add path resolution for shared project config, personal project config, and user config.
- [ ] Add path resolution for adjacent lockfiles.
- [ ] Add generated state path resolution for project and user scopes.
- [ ] Keep repo-root discovery minimal and local; do not add global org/team discovery.
- [ ] Add tests using temporary directories and controlled environment variables.

Validation:

```sh
cargo test -p agentcfg-core config_paths
```

#### Task M1.3: Implement config parsing and validation

- [ ] Parse V1 TOML config into skill-specific structs.
- [ ] Validate `scope` against config location.
- [ ] Validate required `[[skill_sources]].id`.
- [ ] Validate required `[skills].clients`.
- [ ] Keep `exclude` unsupported in V1.
- [ ] Add tests for valid shared, personal, and user configs plus validation failures.

Validation:

```sh
cargo test -p agentcfg-core config
```

#### Task M1.4: Implement `init` file creation

- [ ] Create the correct config file for default, `--project`, and `--user`.
- [ ] Create `.agentcfg/` only when needed.
- [ ] Do not write client target directories.
- [ ] Refuse to overwrite existing config files.
- [ ] Report existing unmanaged client artifacts without adopting them.
- [ ] Add CLI/core tests for each init mode.

Validation:

```sh
cargo test --workspace init
```

### M2: Path Sources and Skill Selection

Goal: resolve selected skills from local path sources without writing managed state.

#### Task M2.1: Discover path-source skill directories

- [ ] Discover direct child directories containing `SKILL.md`.
- [ ] Return original skill names and source paths.
- [ ] Reject missing source directories with a clear diagnostic.
- [ ] Do not support nested layouts beyond the selected V1 source layout until the open question is resolved.
- [ ] Add tests for discovery, empty sources, and missing sources.

Validation:

```sh
cargo test -p agentcfg-core path_source_discovery
```

#### Task M2.2: Resolve `include` selections

- [ ] Select all discovered skills when neither `include` nor `groups` is set.
- [ ] Select only named skills when `include` is set.
- [ ] Report missing included skills with source and layer context.
- [ ] Add tests for all-skills, included-skills, and missing-include cases.

Validation:

```sh
cargo test -p agentcfg-core skill_selection
```

#### Task M2.3: Resolve source-local groups

- [ ] Parse optional source `skills.toml`.
- [ ] Resolve selected `groups` into skill names.
- [ ] Report missing groups and group references to missing skills.
- [ ] Merge `include` and `groups` deterministically.
- [ ] Add tests for valid groups, missing groups, and missing group members.

Validation:

```sh
cargo test -p agentcfg-core source_groups
```

#### Task M2.4: Apply aliases and produce installed identities

- [ ] Apply layer-local aliases after source-local group expansion.
- [ ] Treat installed name as runtime identity.
- [ ] Preserve original source names for lockfile and manifest records.
- [ ] Keep output structured enough for later target-path collision detection.
- [ ] Add tests for alias success, unaliased skills, and original-name preservation.

Validation:

```sh
cargo test -p agentcfg-core aliases
```

### M3: Safe Materialization and Hashing

Goal: produce deterministic managed skill trees from selected source skills.

#### Task M3.1: Implement safe tree walk

- [ ] Walk a skill directory recursively.
- [ ] Normalize relative paths to POSIX-style `/`.
- [ ] Sort entries lexicographically.
- [ ] Reject special files.
- [ ] Detect broken symlinks.
- [ ] Add tests for ordering, nested files, special-file rejection where supported, and broken symlinks.

Validation:

```sh
cargo test -p agentcfg-core materialization_walk
```

#### Task M3.2: Materialize internal symlinks and reject external symlinks

- [ ] Resolve symlinks relative to the skill directory.
- [ ] Materialize internal symlinked files and directories as regular content.
- [ ] Reject symlinks resolving outside the skill directory.
- [ ] Preserve deterministic output independent of source symlink layout.
- [ ] Add tests for internal file symlink, internal directory symlink, and external symlink rejection.

Validation:

```sh
cargo test -p agentcfg-core symlink_materialization
```

#### Task M3.3: Implement deterministic tree hashing

- [ ] Hash length-prefixed normalized paths and length-prefixed content bytes.
- [ ] Return `sha256:<hex>`.
- [ ] Hash the materialized tree, not only `SKILL.md`.
- [ ] Add golden tests for stable hashes and content/path changes.

Validation:

```sh
cargo test -p agentcfg-core hashing
```

#### Task M3.4: Document the alias frontmatter rewrite contract

- [ ] Update `design-v1.md` with the exact `SKILL.md` frontmatter rewrite contract before implementation.
- [ ] Define the supported frontmatter delimiter and name-field behavior.
- [ ] Define behavior when no supported frontmatter is present.
- [ ] Define that source files are never mutated.

Validation:

```sh
git diff -- docs/design-v1.md
```

#### Task M3.5: Apply alias rewrite during materialization

- [ ] Rewrite managed `SKILL.md` frontmatter `name` when an alias is applied.
- [ ] Do not mutate the upstream source.
- [ ] Record both `source_hash` before alias rewrite and `installed_hash` after alias rewrite.
- [ ] Add tests for frontmatter rewrite, no-frontmatter behavior, and hash differences.

Validation:

```sh
cargo test -p agentcfg-core alias_rewrite
```

### M4: Lockfiles and Managed Sources

Goal: make path-source sync repeatable from locked managed state.

#### Task M4.1: Define lockfile models and TOML persistence

- [ ] Model lockfile records for path sources.
- [ ] Include source id, source type, original name, installed name, source hash, installed hash, alias rewrite state, and materialized symlink metadata.
- [ ] Read and write adjacent lockfiles.
- [ ] Preserve deterministic lockfile ordering.
- [ ] Add round-trip and ordering tests.

Validation:

```sh
cargo test -p agentcfg-core lockfile
```

#### Task M4.2: Materialize managed source trees for path sources

- [ ] Write selected installed skill trees under the active scope's managed source directory.
- [ ] Use a stable path containing layer, source id, resolved id, and installed name.
- [ ] Avoid rewriting existing identical managed trees unnecessarily.
- [ ] Add tests for project and user managed source paths.

Validation:

```sh
cargo test -p agentcfg-core managed_sources
```

#### Task M4.3: Implement plain sync locked-source behavior for path sources

- [ ] Reuse locked managed source trees when present.
- [ ] Recreate missing managed trees only when the current source materializes to the locked `source_hash`.
- [ ] Fail when the source is unavailable and managed state is missing.
- [ ] Fail when the source changed from the locked hash and managed state is missing.
- [ ] Add tests for all four cases.

Validation:

```sh
cargo test -p agentcfg-core locked_path_sync
```

#### Task M4.4: Implement path-source upgrade resolution

- [ ] Make `plan --upgrade` refresh path-source hashes in memory only.
- [ ] Make `sync --upgrade` rewrite active lockfiles.
- [ ] Make `sync --upgrade` materialize refreshed managed trees.
- [ ] Verify non-upgrade `plan` and `plan --upgrade` do not write persistent state.
- [ ] Add tests for changed source content and read-only plan behavior.

Validation:

```sh
cargo test --workspace path_upgrade read_only_plan
```

### M5: Client Registry, Planning, and CLI Rendering

Goal: produce structured plans once and render them through the CLI.

#### Task M5.1: Implement built-in client target registry

- [ ] Add V1 default clients and target paths.
- [ ] Represent project and user target paths.
- [ ] Represent confidence/provenance metadata for diagnostics.
- [ ] Resolve shared `.agents/skills/{name}` target paths for Codex, Pi, OpenCode, and Cursor.
- [ ] Add tests for every built-in client target.

Validation:

```sh
cargo test -p agentcfg-core client_registry
```

#### Task M5.2: Build desired target state from active layers

- [ ] Combine shared project and personal project layers for project commands.
- [ ] Use only user config for `--user` commands.
- [ ] Apply additive client selection across active layers.
- [ ] Do not add CLI `--client` unless `prd.md` and `design-v1.md` are updated first.
- [ ] Add tests for project layering, user-only mode, and shared target consumers.

Validation:

```sh
cargo test -p agentcfg-core desired_state
```

#### Task M5.3: Generate structured plan entries

- [ ] Detect installed-name collisions per target path after client target resolution.
- [ ] Merge consumers only when selected entries refer to the same locked source skill and installed hash.
- [ ] Include layer/source context in collision diagnostics.
- [ ] Generate lockfile create/update entries.
- [ ] Generate target create/update entries.
- [ ] Generate consumer addition entries.
- [ ] Generate stale consumer/artifact entries for reporting only.
- [ ] Keep plan records structured and free of terminal formatting.
- [ ] Add tests for create, update, no-op, and stale reporting plans.

Validation:

```sh
cargo test -p agentcfg-core planner
```

#### Task M5.4: Render `plan` output in the CLI

- [ ] Render structured plan entries as human-readable terminal output.
- [ ] Include alias rewrites and uncertain target warnings.
- [ ] Ensure `agentcfg plan` performs no persistent writes.
- [ ] Add CLI snapshot or assertion tests for representative plan output.

Validation:

```sh
cargo test --workspace plan_render
```

### M6: Sync Apply, Manifest, and Prune

Goal: safely mutate only manifest-owned target artifacts.

#### Task M6.1: Define manifest models and JSON persistence

- [ ] Model manifest records with kind, source id, original name, installed name, target path, target kind, installed hash, consumers, created-by marker, source acquisition mode, and target mode.
- [ ] Read and write project and user manifests.
- [ ] Preserve structured consumers by `{scope, client}`.
- [ ] Add round-trip and ordering tests.

Validation:

```sh
cargo test -p agentcfg-core manifest
```

#### Task M6.2: Apply target creates and updates

- [ ] Create client target symlinks to managed source trees.
- [ ] Update manifest-owned symlinks only when the current target matches the manifest.
- [ ] Refuse to overwrite unmanaged files or unexpected symlinks.
- [ ] Add required consumers to manifest records.
- [ ] Warn when stale artifacts remain after sync.
- [ ] Add tests for create, safe update, unmanaged conflict, and unexpected symlink refusal.

Validation:

```sh
cargo test -p agentcfg-core sync_apply
```

#### Task M6.3: Detect stale consumers and artifacts

- [ ] Compare manifest consumers against desired target state.
- [ ] Mark removed layer/client pairs as stale consumers.
- [ ] Mark target artifacts with no remaining consumers as stale artifacts.
- [ ] Detect unused managed source trees as cache leftovers.
- [ ] Add tests for shared `.agents/skills` consumers across Codex, Pi, OpenCode, and Cursor.

Validation:

```sh
cargo test -p agentcfg-core stale_state shared_consumers
```

#### Task M6.4: Implement prune safety engine

- [ ] Remove stale consumers.
- [ ] Remove target artifacts only when no consumers remain.
- [ ] Refuse to prune unexpected symlink targets.
- [ ] Never delete unmanaged real files.
- [ ] Delete directories only if empty and manifest-owned.
- [ ] Add tests for each safety invariant.

Validation:

```sh
cargo test -p agentcfg-core prune
```

### M7: Status and Doctor

Goal: expose local consistency and environment diagnostics without duplicating planner logic.

#### Task M7.1: Implement structured status checks

- [ ] Report installed managed artifacts by client.
- [ ] Report broken symlinks and unexpected targets.
- [ ] Report missing managed sources.
- [ ] Report stale artifacts and cache leftovers.
- [ ] Report config/lock mismatch.
- [ ] Report unmanaged artifacts as informational unless they conflict.
- [ ] Add tests using temporary manifests and target directories.

Validation:

```sh
cargo test -p agentcfg-core status
```

#### Task M7.2: Render `status` in the CLI

- [ ] Render structured status results.
- [ ] Use script-friendly output conventions where practical.
- [ ] Map inconsistent state to the intended exit code.
- [ ] Add CLI output tests for clean and inconsistent states.

Validation:

```sh
cargo test --workspace status_render
```

#### Task M7.3: Implement structured doctor checks

- [ ] Check git availability.
- [ ] Check repo root detection.
- [ ] Check supported clients and target confidence.
- [ ] Check path writability.
- [ ] Check config schema validity.
- [ ] Check unmanaged artifacts only when they block planned target paths.
- [ ] Keep optional network/source checks isolated so local doctor remains deterministic in tests.
- [ ] Add tests with injectable command/path probes.

Validation:

```sh
cargo test -p agentcfg-core doctor
```

#### Task M7.4: Render `doctor` in the CLI

- [ ] Render structured doctor results.
- [ ] Distinguish errors, warnings, and informational diagnostics.
- [ ] Map blocking diagnostics to the intended exit code.
- [ ] Add CLI output tests for passing and failing diagnostics.

Validation:

```sh
cargo test --workspace doctor_render
```

### M8: Git Sources

Goal: add git sources by reusing the path-source discovery, materialization, hashing, lockfile, and planner pipeline.

#### Task M8.1: Model git source config and validation

- [ ] Extend source config parsing for `type = "git"`.
- [ ] Validate required git fields.
- [ ] Preserve requested ref separately from resolved commit.
- [ ] Add parsing and validation tests.

Validation:

```sh
cargo test -p agentcfg-core git_config
```

#### Task M8.2: Resolve git sources into local source trees

- [ ] Clone or fetch git sources into an internal cache.
- [ ] Resolve floating refs to concrete commits.
- [ ] Support pinned commit refs without treating them as floating.
- [ ] Keep network/git command execution behind an injectable boundary for tests.
- [ ] Add tests using local fixture repositories.

Validation:

```sh
cargo test -p agentcfg-core git_resolution
```

#### Task M8.3: Discover and materialize git skills through the existing pipeline

- [ ] Run skill discovery against resolved git content.
- [ ] Reuse source-local group resolution.
- [ ] Reuse safe materialization and hashing.
- [ ] Reuse alias rewrite behavior.
- [ ] Add tests proving path and git sources produce equivalent installed trees for equivalent content.

Validation:

```sh
cargo test -p agentcfg-core git_materialization
```

#### Task M8.4: Implement locked git sync behavior

- [ ] Reuse locked managed git source trees for plain `sync`.
- [ ] Recreate missing managed trees from locked commits when available.
- [ ] Fail clearly when a locked commit cannot be fetched or restored.
- [ ] Add tests using local fixture repositories.

Validation:

```sh
cargo test -p agentcfg-core locked_git_sync
```

#### Task M8.5: Implement git upgrade behavior

- [ ] Make `plan --upgrade` detect moved floating refs without persistent writes.
- [ ] Make `sync --upgrade` update lockfiles to resolved commits.
- [ ] Make `sync --upgrade` materialize refreshed managed trees.
- [ ] Add tests for floating ref movement and pinned commit no-op.

Validation:

```sh
cargo test -p agentcfg-core git_upgrade
```

## End-to-End Validation

Run after each milestone:

```sh
cargo test --workspace
```

Run before declaring V1 complete:

- [ ] `agentcfg init` creates `.agentcfg/config.toml`.
- [ ] `agentcfg init --project` creates `agentcfg.toml`.
- [ ] `agentcfg init --user` creates `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`.
- [ ] `agentcfg plan` performs no persistent writes.
- [ ] `agentcfg sync` installs a path-source skill from locked managed state.
- [ ] `agentcfg sync --upgrade` imports changed path-source content.
- [ ] `agentcfg prune` removes only manifest-owned stale artifacts.
- [ ] Alias collision behavior is covered.
- [ ] Internal symlink materialization and external symlink rejection are covered.
- [ ] Shared `.agents/skills` consumers across Codex/Pi/OpenCode/Cursor are covered.
- [ ] Git source sync and upgrade behavior are covered by local fixture repositories.

## Open Planning Questions

- [X] Should git sources be included before or after the first path-source sync milestone? YES.
- [ ] How much source provenance should be exposed in normal command output versus `doctor`?
- [ ] Should V1 support both `skills/<name>/SKILL.md` and root-level `<name>/SKILL.md` source layouts?
