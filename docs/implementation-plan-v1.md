# agentcfg V1 Implementation Plan

This document breaks V1 into implementation milestones. Product intent lives in [prd.md](prd.md); technical contracts live in [design-v1.md](design-v1.md). If this plan conflicts with `design-v1.md`, treat `design-v1.md` as authoritative and update the plan.

The plan is optimized for agent execution. Each task should be small enough that an implementation agent has at least 90% confidence it can complete the task from this document, the PRD, and the design doc without needing another design discussion.

## Agent Task Rules

- Keep each task focused on one ownership boundary.
- Prefer tasks that produce a compiling, testable state.
- Do not mix domain logic, persistence, filesystem mutation, and CLI rendering in one task unless the task is explicitly an end-to-end slice.
- Add tests in the same task as the behavior unless the task is scaffolding only.
- If a task exposes a hidden design decision, stop and update `design-v1.md` before implementing.
- Keep the core crate skill-first. Share target planning/apply/status/prune around structured desired target artifacts, but do not introduce generic resource manager traits, factories, or interfaces before a second resource kind exists.
- Treat CLI command handlers as adapters into core workflow APIs. As lower-level behavior is implemented, expose focused core APIs for config paths, config parsing, source/skill resolution, desired target state, planning, apply/prune safety, status, and doctor checks. The CLI should not orchestrate those lower-level steps directly.
- If a future resource-kind or resource-specific CLI selector question appears during V1 work, record it in the post-V1 holding area in `design-v1.md` instead of expanding V1 scope.

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

- [x] Add a core `Result<T>` alias and error type.
- [x] Add CLI error-to-exit-code mapping.
- [x] Keep terminal formatting out of core errors except concise diagnostic strings.
- [x] Add tests for at least one CLI error mapping.

Validation:

```sh
cargo test --workspace
```

### M1: CLI Surface and Config Paths

Goal: make every V1 command invocable while keeping behavior stubbed until core workflows and config path APIs exist.

#### Task M1.1: Define CLI command surface

- [x] Introduce `clap` for command parsing instead of growing manual argument parsing.
- [x] Add `init`, `preview` (formerly `plan`), `apply` (formerly `sync`), `prune`, `status`, and `doctor`.
- [x] Add `--project`, `--user`, and `--refresh-sources` only where allowed by the PRD.
- [x] Reject invalid flag combinations through argument parsing where possible.
- [x] Map parser usage errors through the M0.2 CLI error adapter to exit code `2`.
- [x] Route each command to a small CLI handler that calls a core workflow stub using structured request/result types.
- [x] Keep workflow APIs namespaced under `agentcfg_core::workflow`; do not root-re-export every stub type before behavior exists.
- [x] Mark public workflow request/result structs `#[non_exhaustive]` when later fields are plausible; do not mark stable domain enums non-exhaustive without a concrete reason.
- [x] Introduce shared `ConfigLayer`, `InstallLevel`, and `SourceResolutionPolicy` types for later core tasks to reuse.
- [x] Keep M1.1 workflow stubs thin; do not introduce speculative lower-level planner/apply APIs before real behavior exists.
- [x] Add CLI snapshot or assertion tests for supported and rejected command forms, including at least one full binary usage-error path.

Validation:

```sh
cargo test --workspace
cargo run -p agentcfg-cli -- preview --help
```

#### Task M1.2: Model config layers, install levels, and paths in core

- [x] Reuse the shared config layer values introduced in M1.1: `shared-project`, `user-project`, and `user`.
- [x] Add path resolution for shared project config, user project config, and user config.
- [x] Add path resolution for adjacent lockfiles.
- [x] Add managed state path resolution for project and user Install Levels.
- [x] Keep repo-root discovery minimal and local; do not add global org/team discovery.
- [x] Expose a focused lower-level config path API that later workflow code can call without going through CLI command types.
- [x] Add tests using temporary directories and controlled environment variables.

Validation:

```sh
cargo test -p agentcfg-core config_paths
```

#### Task M1.3: Implement config parsing and validation

- [x] Add a focused `agentcfg_core::config` module for V1 skill config models, parsing, loading, and validation.
- [x] Centralize persisted `ConfigLayer` scope strings (`shared-project`, `user-project`, `user`) so parsing, diagnostics, lockfiles, and manifests reuse one contract.
- [x] Parse V1 TOML config into skill-specific structs.
- [x] Validate `scope` against config location.
- [x] Validate required `[[skill_sources]].id`.
- [x] Validate required `[skills].clients`; accept either a non-empty explicit client list or `clients = "all"` for all supported clients.
- [x] Keep `exclude` unsupported in V1.
- [x] Add structured config validation errors for parse failures, scope mismatch, missing source id, missing clients, and unsupported fields; include enough path/layer/field context for CLI diagnostics without embedding CLI formatting in core.
- [x] Expose lower-level config load/parse/validate APIs returning structured config models for the active layer types.
- [x] Add tests for persisted scope string mapping, valid shared, user project, and user configs, and validation failures.

Validation:

```sh
cargo test -p agentcfg-core config
```

#### Task M1.4: Implement `init` file creation

- [x] Introduce an internal workflow execution context or `init_with_context` helper so cwd, user dirs, project-root discovery, and filesystem effects are injectable in tests and not read ad hoc inside public workflow entrypoints.
- [x] Create the correct config file for default, `--project`, and `--user`.
- [x] Create `.agentcfg/` only when needed.
- [x] Do not write Client Discovery Location directories.
- [x] Refuse to overwrite existing config files.
- [x] Report existing unmanaged Installed Artifacts without adopting them.
- [x] Implement `init` as a core workflow that composes config path APIs with conservative file creation.
- [x] Add CLI/core tests for each init mode.

Validation:

```sh
cargo test --workspace init
```

### M1.5: Ubiquitous Language Alignment

Goal: align command names, core API names, persisted models, and domain docs with the root ubiquitous language document before more preview/apply work lands.

Before starting M2, update this implementation plan's downstream milestones so new work does not copy pre-glossary terms.

#### Task M1.5.0: Rename plan workflow language to preview

- [x] Rename the user-facing `plan` workflow to `preview`, including CLI command, help text, workflow request/result names, tests, and docs.
- [x] Preserve the strict read-only invariant: preview never writes config, lockfiles, manifests, managed state, source locations, or Client Discovery Locations.
- [x] Decide whether `plan` remains as a temporary compatibility alias or is removed before V1 release. **Decision:** remove `plan` subcommand; no compatibility alias (M1.5.0).
- [x] Update validation commands and test names that currently use `plan`.

Validation:

```sh
cargo test --workspace preview
```

#### Task M1.5.1: Rename sync workflow language to apply

- [x] Rename the user-facing `sync` workflow to `apply`, including CLI command, help text, workflow request/result names, tests, and docs.
- [x] Decide whether `sync` remains as a temporary compatibility alias or is removed before V1 release. **Decision:** remove `sync` subcommand; no compatibility alias (M1.5.1).
- [x] Preserve the one-way invariant: apply writes managed state and Client Discovery Locations, never source locations.
- [x] Update validation commands and test names that currently use `sync`.

Validation:

```sh
cargo test --workspace apply
```

#### Task M1.5.2: Align config layer and install level language

- [x] Keep `ConfigLayer` as the core type for `shared-project`, `user-project`, and `user` Config Layers.
- [x] Align Active Config Layers wording so Project Level means Shared Project Config then User Project Config, and User Level means User Config only.
- [x] Rename `InstallScope` language to Install Level in domain docs, CLI help, workflow APIs, diagnostics, and tests.
- [x] Align Project, Project Root, User, Project Level, and User Level wording in path discovery, diagnostics, and CLI help.
- [x] Keep persisted `scope = ...` wording distinct as Persisted Scope Value in config parsing and diagnostics.
- [x] Avoid override language for V1 Project Level behavior; User Project Config is additive with Shared Project Config.
- [x] Update `--user` help to say it selects User Config for `init` and the user Install Level for preview/apply/prune/status.

Validation:

```sh
cargo test --workspace config_layer install_level
```

#### Task M1.5.3: Align discovery, artifact, and requirement terms

- [x] Rename domain docs from client target/target registry language to Client Discovery Location and Client Discovery Registry.
- [x] Keep implementation path types only as low-level structures when the name is still useful; do not expose target language in user-facing diagnostics.
- [x] Rename Consumer model/docs to Discovery Requirement.
- [x] Rename target artifact/user-facing artifact language to Installed Artifact.
- [x] Update manifest and planner terminology from consuming `{scope, client}` pairs to Discovery Requirements keyed by Config Layer, Client, and Install Level.

Validation:

```sh
cargo test --workspace discovery_registry
```

#### Task M1.5.4: Align skill source, selection, and managed content terms

- [x] Rename standard/Agent Skills Standard wording to Agent Skill Format where referring to the `SKILL.md` directory format.
- [x] Rename Source/domain-doc wording to Skill Source for V1 skill acquisition.
- [x] Keep Source Location out of canonical API/model names until multiple Configured Item kinds prove they share a source-resolution lifecycle.
- [x] Rename Managed Source Tree/copy wording to Managed Skill Content, including lockfile, materialization, and status docs.
- [x] Rename installed name/runtime identity wording to Discovery Name; keep Source Skill Name for source identity.
- [x] Rename alias/installed-name collision wording to Discovery Name Collision.
- [x] Align include/group docs with domain-shaped terms: Skill Selection, Included Skill, and Skill Group.
- [x] Update alias docs to say Skill Alias changes the Discovery Name and may require Managed Skill Content frontmatter preparation.
- [x] Rename upgrade wording to Source Refresh for source-resolution refresh behavior, including CLI flag `--refresh-sources`, workflow APIs, tests, and docs.

Validation:

```sh
cargo test --workspace skill_source skill_selection discovery_name source_refresh
```

#### Task M1.5.5: Align desired-state, lockfile, manifest, and managed-state terms

- [x] Introduce Configured Item as the shared term for item kinds managed by `agentcfg`; keep V1 skill-specific code skill-specific until another kind exists.
- [x] Align Desired State and Locked Desired State wording in planner, lockfile, preview, and apply docs.
- [x] Align Lockfile wording to record Locked Desired State for Configured Items that need repeatable source resolution.
- [x] Align Manifest wording as the ownership state for Installed Artifacts and their Discovery Requirements.
- [x] Rename generated/cache/internal-state wording to Managed State where referring to `agentcfg`-owned state used by apply, status, and prune.

Validation:

```sh
cargo test --workspace lockfile manifest
cargo test --workspace
rg 'GeneratedStatePaths|generated state' crates/ docs/prd.md docs/design-v1.md README.md
```

#### Task M1.5.6: Align status, prune, and safety terminology

- [x] Use Unmanaged Artifact for filesystem entries at Client Discovery Locations that are not recorded in the Manifest.
- [x] Use Stale Discovery Requirement for Manifest requirements no longer present in Desired State.
- [x] Use Unsatisfied Discovery Requirement for Desired State requirements without a valid Installed Artifact.
- [x] Use Stale Installed Artifact for Manifest-recorded Installed Artifacts with no remaining Discovery Requirements.
- [x] Keep Unexpected Symlink Target and Broken Symlink scoped to filesystem symlink diagnostics, not client-target language.
- [x] Preserve Status as managed install-state consistency and Doctor as environment/configuration readiness.

Validation:

```sh
cargo test --workspace
rg 'UnmanagedInstalledArtifact|unmanaged installed artifact' crates/ docs/prd.md docs/design-v1.md README.md
rg -i 'stale consumer|broken target' docs/prd.md docs/design-v1.md README.md
```

#### Task M1.5.7: Update downstream milestone wording

- [ ] Update M2 and later milestones in this plan to use the root ubiquitous language document before implementing those milestones.
- [ ] Replace pre-glossary terms in downstream task names, checklists, validation commands, and acceptance notes.
- [ ] Keep implementation-only names only where the task explicitly calls out a low-level structure that intentionally differs from domain language.

Validation:

```sh
rg "sync|plan|upgrade|InstallScope|Consumer|installed name|target registry|Managed Source Tree" docs/implementation-plan-v1.md
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
- [ ] Keep selection output structured for later aliasing, materialization, and desired-target construction.
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
- [ ] Expose the skill resolution output as a lower-level core API, not a CLI-rendered summary.
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
- [ ] Thread `SourceResolutionPolicy` into lower-level resolution APIs without using CLI flag-shaped booleans.
- [ ] Verify non-upgrade `plan` and `plan --upgrade` do not write persistent state.
- [ ] Add tests for changed source content and read-only plan behavior.

Validation:

```sh
cargo test --workspace path_upgrade read_only_plan
```

### M5: Client Registry, Planning, and CLI Rendering

Goal: produce structured plans once and render them through the CLI.

#### Task M5.1: Implement built-in client target registry

- [ ] Add V1 default clients and skill target paths.
- [ ] Key registry entries by `{resource_kind, client, scope}` with only `resource_kind = "skill"` in V1.
- [ ] Represent project and user target paths.
- [ ] Represent confidence/provenance metadata for diagnostics.
- [ ] Resolve shared `.agents/skills/{name}` target paths for Codex, Pi, OpenCode, and Cursor.
- [ ] Resolve Cline through `.cline/skills/{name}` and `~/.cline/skills/{name}` with experimental provenance.
- [ ] Do not model shared `.agents` support as a client-family interface; use multiple client entries that resolve to the same path.
- [ ] Add tests for every built-in client target.

Validation:

```sh
cargo test -p agentcfg-core client_registry
```

#### Task M5.2: Build desired target state from active layers

- [ ] Combine shared project and user project layers for project commands.
- [ ] Use only user config for `--user` commands.
- [ ] Apply additive client selection across active layers.
- [ ] Add repeatable CLI `--client` for `plan`, `sync`, `prune`, and `status`; carry it through workflow requests as a client filter.
- [ ] Treat omitted `--client` as all clients selected by active config layers.
- [ ] Validate that each requested `--client` is both a known V1 client and selected by the active config; when config uses `clients = "all"`, allow any supported V1 client. Do not let CLI flags add clients outside the configured selection.
- [ ] Convert resolved skills into structured desired target artifacts before planning target changes.
- [ ] Include kind, target path, target mode, managed source path, installed name, installed hash, source/layer provenance, and consuming `{scope, client}` pairs in desired target artifacts.
- [ ] Expose desired target state construction as a lower-level core API that `plan`, `sync`, `status`, and `prune` can share.
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
- [ ] Keep planner records resource-kind aware but skill-first; do not add generic resource manager interfaces.
- [ ] Expose the planner as a lower-level core API that consumes desired target artifacts and current lock/manifest state.
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

- [ ] Model manifest records with resource kind, source id, original name, installed name, target path, target kind, installed hash, consumers, created-by marker, source acquisition mode, and target mode.
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
- [ ] Expose sync apply as a lower-level core API that consumes structured plan entries; keep terminal warnings in the CLI renderer.
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
- [ ] Expose stale-state detection as a lower-level core API shared by plan reporting and prune.
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
- [ ] Expose prune apply as a lower-level core API that consumes stale-state records.
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
- [ ] Expose status checks as structured core results so CLI rendering stays separate.
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
- [ ] Expose doctor checks as structured core results with severity and context; do not return terminal-formatted text from core.
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
- [ ] Cline native `.cline/skills` target behavior is covered with experimental provenance.
- [ ] Git source sync and upgrade behavior are covered by local fixture repositories.

## Open Planning Questions

- [X] Should git sources be included before or after the first path-source sync milestone? YES.
- [ ] How much source provenance should be exposed in normal command output versus `doctor`?
- [ ] Should V1 support both `skills/<name>/SKILL.md` and root-level `<name>/SKILL.md` source layouts?
