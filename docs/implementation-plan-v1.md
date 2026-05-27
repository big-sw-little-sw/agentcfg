# agentcfg V1 Implementation Plan

This document breaks V1 into implementation milestones. Product intent lives in [prd.md](prd.md); technical contracts live in [design-v1.md](design-v1.md). If this plan conflicts with `design-v1.md`, stop and ask which source to follow for that change; do not assume one document wins over the other.

The plan is optimized for agent execution. Each task should be small enough that an implementation agent has at least 90% confidence it can complete the task from this document, the PRD, and the design doc without needing another design discussion.

## Agent Task Rules

- Keep each task focused on one ownership boundary.
- Prefer tasks that produce a compiling, testable state.
- Do not mix domain logic, persistence, filesystem mutation, and CLI rendering in one task unless the task is explicitly an end-to-end slice.
- Add tests in the same task as the behavior unless the task is scaffolding only.
- If a task exposes a hidden design decision, stop and ask or update `design-v1.md` before implementing.
- If `design-v1.md` and this plan disagree on the same behavior, stop and ask; do not pick one silently.
- Keep the core crate skill-first. Share preview/apply/status/prune around structured Desired State for Installed Artifacts, but do not introduce generic Configured Item manager traits, factories, or interfaces before a second Configured Item kind exists.
- Treat CLI command handlers as adapters into core workflow APIs. As lower-level behavior is implemented, expose focused core APIs for config paths, config parsing, Skill Source resolution, Skill Selection, Desired State, preview operation generation, apply/prune safety, status, and doctor checks. The CLI should not orchestrate those lower-level steps directly.
- If a future Configured Item-kind-specific CLI selector question appears during V1 work, record it in the post-V1 holding area in `design-v1.md` instead of expanding the V1 boundary.

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
- `agentcfg-core`: config, Skill Source discovery, materialization, hashing, lockfiles, manifests, preview operation generation, apply operations, status, doctor, and Client Discovery Registry.

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
- [x] Introduce shared `ConfigLayer`, `InstallLevel`, and `SkillSourceResolutionPolicy` types for later core tasks to reuse.
- [x] Keep M1.1 workflow stubs thin; do not introduce speculative lower-level preview/apply operation APIs before real behavior exists.
- [x] Add CLI snapshot or assertion tests for supported and rejected command forms, including at least one full binary usage-error path.

Validation:

```sh
cargo test --workspace
cargo run -p agentcfg-cli -- preview --help
```

#### Task M1.2: Model config layers, install levels, and paths in core

- [x] Reuse the shared config layer values introduced in M1.1: `shared-project`, `user-project`, and `user`.
- [x] Add path resolution for Shared Project Config, User Project Config, and User Config.
- [x] Add path resolution for adjacent lockfiles.
- [x] Add Managed State path resolution for project and user Install Levels.
- [x] Keep Project Root discovery minimal and local; do not add global org/team discovery.
- [x] Expose a focused lower-level config path API that later workflow code can call without going through CLI command types.
- [x] Add tests using temporary directories and controlled environment variables.

Validation:

```sh
cargo test -p agentcfg-core config_paths
```

#### Task M1.3: Implement config parsing and validation

- [x] Add a focused `agentcfg_core::config` module for V1 skill config models, parsing, loading, and validation.
- [x] Centralize Persisted Scope Value strings (`shared-project`, `user-project`, `user`) so parsing, diagnostics, lockfiles, and manifests reuse one contract.
- [x] Parse V1 TOML config into skill-specific structs.
- [x] Validate `scope` against config location.
- [x] Validate required `[[skill_sources]].id`.
- [x] Validate required `[skills].clients`; accept either a non-empty explicit client list or `clients = "all"` for all supported clients.
- [x] Keep `exclude` unsupported in V1.
- [x] Add structured config validation errors for parse failures, Persisted Scope Value mismatch, missing Skill Source id, missing clients, and unsupported fields; include enough path/layer/field context for CLI diagnostics without embedding CLI formatting in core.
- [x] Expose lower-level config load/parse/validate APIs returning structured config models for the active layer types.
- [x] Add tests for Persisted Scope Value mapping, valid Shared Project Config, User Project Config, and User Config, and validation failures.

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
- [x] Report existing Unmanaged Artifacts without adopting them.
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
- [x] Preserve the strict read-only invariant: preview never writes config, lockfiles, manifests, Managed State, Skill Sources, or Client Discovery Locations.
- [x] Decide whether `plan` remains as a temporary compatibility alias or is removed before V1 release. **Decision:** remove `plan` subcommand; no compatibility alias (M1.5.0).
- [x] Update validation commands and test names that currently use `plan`.

Validation:

```sh
cargo test --workspace preview
```

#### Task M1.5.1: Rename sync workflow language to apply

- [x] Rename the user-facing `sync` workflow to `apply`, including CLI command, help text, workflow request/result names, tests, and docs.
- [x] Decide whether `sync` remains as a temporary compatibility alias or is removed before V1 release. **Decision:** remove `sync` subcommand; no compatibility alias (M1.5.1).
- [x] Preserve the one-way invariant: apply writes Managed State and Client Discovery Locations, never Skill Sources.
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
- [x] Update manifest and preview operation terminology from consuming `{scope, client}` pairs to Discovery Requirements keyed by Config Layer, Client, and Install Level.

Validation:

```sh
cargo test --workspace discovery_registry
```

#### Task M1.5.4: Align skill source, selection, and managed content terms

- [x] Rename standard/Agent Skills Standard wording to Agent Skill Format where referring to the `SKILL.md` directory format.
- [x] Rename Source/domain-doc wording to Skill Source for V1 skill acquisition.
- [x] Keep Source Location out of canonical API/model names until multiple Configured Item kinds prove they share the same external-origin resolution lifecycle.
- [x] Rename Managed Source Tree/copy wording to Managed Skill Content, including lockfile, materialization, and status docs.
- [x] Rename installed name/runtime identity wording to Discovery Name; keep Source Skill Name for source identity.
- [x] Rename alias/installed-name collision wording to Discovery Name Collision.
- [x] Align include/group docs with domain-shaped terms: Skill Selection, Included Skill, and Skill Group.
- [x] Update alias docs to say Skill Alias changes the Discovery Name and may require Managed Skill Content frontmatter preparation.
- [x] Rename upgrade wording to Source Refresh for Skill Source resolution refresh behavior, including CLI flag `--refresh-sources`, workflow APIs, tests, and docs.

Validation:

```sh
cargo test --workspace skill_source skill_selection discovery_name source_refresh
```

#### Task M1.5.5: Align desired-state, lockfile, manifest, and managed-state terms

- [x] Introduce Configured Item as the shared term for item kinds managed by `agentcfg`; keep V1 skill-specific code skill-specific until another kind exists.
- [x] Align Desired State and Locked Desired State wording in preview operation, lockfile, preview, and apply docs.
- [x] Align Lockfile wording to record Locked Desired State for Configured Items that need repeatable Skill Source resolution.
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
- [x] Keep Unexpected Symlink Target and Broken Symlink limited to filesystem symlink diagnostics, not client-target language.
- [x] Preserve Status as managed install-state consistency and Doctor as environment/configuration readiness.

Validation:

```sh
cargo test --workspace
rg 'UnmanagedInstalledArtifact|unmanaged installed artifact' crates/ docs/prd.md docs/design-v1.md README.md
rg -i 'stale consumer|broken target' docs/prd.md docs/design-v1.md README.md
```

#### Task M1.5.7: Update downstream milestone wording

- [x] Update M2 and later milestones in this plan to use the root ubiquitous language document before implementing those milestones.
- [x] Replace pre-glossary terms in downstream task names, checklists, validation commands, and acceptance notes.
- [x] Keep implementation-only names only where the task explicitly calls out a low-level structure that intentionally differs from domain language.

Validation:

```sh
rg <terminology-audit-patterns> docs/implementation-plan-v1.md
```

### M1.6: Pre-M2 foundations

Goal: stable contributor/agent onboarding, module seams for M2+, and honest CLI behavior before Skill Source discovery work.

#### Task M1.6.1: Toolchain and agent docs

- [x] Pin Rust in `rust-toolchain.toml`.
- [x] Add stable [AGENTS.md](../AGENTS.md) (no milestone-specific status; point to README § Status).
- [x] Symlink [CLAUDE.md](../CLAUDE.md) → `AGENTS.md`.
- [x] Update [README.md](../README.md): implementation status table, concepts → code map, doc links.

Validation:

```sh
rustup show active-toolchain
test -L CLAUDE.md && test "$(readlink CLAUDE.md)" = AGENTS.md
```

#### Task M1.6.2: Core module seams

- [x] Split `workflow` into `init`, `context`, and `types` submodules.
- [x] Move `config` unit tests to `config/tests.rs` (no extra indentation in that file).
- [x] Add placeholder modules with module docs only: `desired_state` (`ConfiguredItemKind`, `NamespacedSkillSourceId`), `lockfile`, `manifest`, `install_health`, `skill_source`.
- [x] `WorkflowContext` resolves paths via `UserDirs` (`config_home`, `state_home`, and `home_dir` for user-level discovery scans).

Validation:

```sh
cargo test --workspace desired_state namespaced_skill_source
```

#### Task M1.6.3: Workflow stubs until M5/M6

- [x] `preview`, `apply`, `prune`, `status`, and `doctor` return `UnsupportedError::Feature` (exit 1 via CLI) until implemented.
- [x] Do not add tests that only assert “not implemented”; remove them when real behavior lands.

#### Task M1.6.4: Workflow split follow-ups (M2 prerequisites)

- [ ] Implement path Skill Source discovery under `skill_source/` (M2.1) — do not grow `workflow::init` with resolution logic.
- [ ] Add `--client` when starting desired-state / preview work (see M5.2 in this plan); PRD documents the flag before CLI exposes it.
- [ ] Validate explicit `skills.clients` against the Client Discovery Registry when client resolution lands (M5.2).
- [x] Tighten init Unmanaged Artifact scan to skill-shaped entries (directories containing `SKILL.md`); warn when user init cannot scan user-level Client Discovery Locations (`HOME` unset).

### M2: Path Skill Sources and Skill Selection

Goal: resolve Skill Selection from local path Skill Sources without writing Managed State.

#### Task M2.1: Discover path Skill Source skill directories

- [ ] Implement discovery under `skill_source/` (do not grow `workflow::init` with resolution logic).
- [ ] Add optional `[[skill_sources]].discovery_depth` (default `4`, max `8`, per Skill Source).
- [ ] Resolve configured `path` relative to the config file’s parent directory, or use absolute paths as-is; discover from the resolved Skill Source directory.
- [ ] Discover **Skills** as directories containing `SKILL.md` within `discovery_depth` path segments below the Skill Source root; skip hidden directories (name starts with `.`).
- [ ] Skip symlink directory entries below the Skill Source root for M2.1; symlink materialization and external symlink rejection are handled in M3.
- [ ] Apply nested-skill exclusion: when a directory contains `SKILL.md`, treat it as a **Skill** and do not scan its children.
- [ ] Set **Source Skill Name** to the skill directory’s leaf name; fail with a clear diagnostic when duplicate leaf names appear at different paths in the same Skill Source.
- [ ] Return structured discoveries (`source_skill_name`, skill directory path); empty Skill Source directories return success with zero skills.
- [ ] Reject missing and non-directory Skill Source paths with distinct clear diagnostics (`skill_source_id`, configured path, resolved path).
- [ ] Document traversal rules in `design-v1.md` § Skill Source Discovery.
- [ ] Add tests for discovery, depth limits, nested-skill exclusion, duplicate names, hidden dirs, symlink directory skipping, empty Skill Sources, missing Skill Sources, non-directory Skill Sources, and path resolution.

Validation:

```sh
cargo test -p agentcfg-core path_skill_source_discovery
```

#### Task M2.2: Resolve Included Skills

- [ ] Select all discovered skills when neither `include` nor `groups` is set.
- [ ] Select only Included Skills when `include` is set.
- [ ] Report missing Included Skills with Skill Source and Config Layer context.
- [ ] Emit a structured warning (CLI renders later) when discovery returns zero skills but the resolved Skill Source directory exists; include `skill_source_id`, resolved path, and effective `discovery_depth`.
- [ ] Keep Skill Selection output structured for later Skill Alias handling, materialization, and Desired State construction.
- [ ] Add tests for all-skills, included-skills, missing-include, and empty-discovery warning cases.

Validation:

```sh
cargo test -p agentcfg-core skill_selection
```

#### Task M2.3: Resolve Skill Source-local Skill Groups

- [ ] Parse optional `skills.toml` only at the resolved Skill Source root (not under organizational subdirectories).
- [ ] Resolve selected `groups` into Source Skill Names.
- [ ] Report missing Skill Groups and Skill Group references to missing Source Skill Names.
- [ ] Merge `include` and `groups` deterministically.
- [ ] Add tests for valid Skill Groups, missing Skill Groups, and missing Skill Group members.

Validation:

```sh
cargo test -p agentcfg-core skill_source_groups
```

#### Task M2.4: Apply Skill Aliases and produce Discovery Names

- [ ] Apply layer-local Skill Aliases after Skill Source-local Skill Group expansion.
- [ ] Treat Discovery Name as the discoverable identity for Clients.
- [ ] Preserve Source Skill Names for lockfile and manifest records.
- [ ] Keep output structured enough for later Discovery Name Collision detection at Client Discovery Locations.
- [ ] Expose Skill Selection output with Discovery Names as a lower-level core API, not a CLI-rendered summary.
- [ ] Add tests for Skill Alias success, unaliased skills, and Source Skill Name preservation.

Validation:

```sh
cargo test -p agentcfg-core discovery_name skill_alias
```

### M3: Safe Materialization and Hashing

Goal: produce deterministic Managed Skill Content from Skill Selection output from Skill Sources.

#### Task M3.1: Implement safe tree walk

- [ ] Walk a skill directory recursively.
- [ ] Normalize relative paths to POSIX-style `/`.
- [ ] Sort entries lexicographically.
- [ ] Reject special files.
- [ ] Detect Broken Symlinks.
- [ ] Add tests for ordering, nested files, special-file rejection where supported, and Broken Symlinks.

Validation:

```sh
cargo test -p agentcfg-core materialization_walk
```

#### Task M3.2: Materialize internal symlinks and reject external symlinks

- [ ] Resolve symlinks relative to the skill directory.
- [ ] Materialize internal symlinked files and directories as regular content.
- [ ] Reject symlinks resolving outside the skill directory.
- [ ] Preserve deterministic output independent of Skill Source symlink layout.
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

#### Task M3.4: Document the Skill Alias frontmatter preparation contract

- [ ] Update `design-v1.md` with the exact `SKILL.md` frontmatter preparation contract before implementation.
- [ ] Define the supported frontmatter delimiter and name-field behavior.
- [ ] Define behavior when no supported frontmatter is present.
- [ ] Define that Skill Source files are never mutated.

Validation:

```sh
git diff -- docs/design-v1.md
```

#### Task M3.5: Apply Discovery Name preparation during materialization

- [ ] Rewrite managed `SKILL.md` frontmatter `name` when a Skill Alias is applied.
- [ ] Do not mutate the upstream Skill Source.
- [ ] Record both `source_hash` before Discovery Name preparation and `installed_hash` after Discovery Name preparation.
- [ ] Add tests for frontmatter preparation, no-frontmatter behavior, and hash differences.

Validation:

```sh
cargo test -p agentcfg-core discovery_name_preparation
```

### M4: Lockfiles and Managed Skill Content

Goal: make path Skill Source apply repeatable from Locked Desired State in Managed State.

#### Task M4.1: Define lockfile models and TOML persistence

- [ ] Model lockfile records for path Skill Sources.
- [ ] Include Skill Source id, Skill Source type, Source Skill Name, Discovery Name, Skill Source hash, installed hash, Discovery Name preparation state, and materialized symlink metadata.
- [ ] Read and write adjacent lockfiles.
- [ ] Preserve deterministic lockfile ordering.
- [ ] Add round-trip and ordering tests.

Validation:

```sh
cargo test -p agentcfg-core lockfile
```

#### Task M4.2: Materialize Managed Skill Content for path Skill Sources

- [ ] Write Skill trees from Skill Selection as Managed Skill Content under the active Install Level's Managed State directory.
- [ ] Use a stable path containing Config Layer, Skill Source id, Skill Source hash, and Discovery Name.
- [ ] Avoid rewriting existing identical Managed Skill Content unnecessarily.
- [ ] Add tests for project and user Managed Skill Content paths.

Validation:

```sh
cargo test -p agentcfg-core managed_skill_content
```

#### Task M4.3: Implement plain apply behavior from Locked Desired State for path Skill Sources

- [ ] Reuse locked Managed Skill Content when present.
- [ ] Recreate missing Managed Skill Content only when the current Skill Source materializes to the locked `source_hash`.
- [ ] Fail when the Skill Source is unavailable and Managed State is missing.
- [ ] Fail when the Skill Source changed from the locked hash and Managed State is missing.
- [ ] Add tests for all four cases.

Validation:

```sh
cargo test -p agentcfg-core locked_path_apply
```

#### Task M4.4: Implement path Skill Source Source Refresh resolution

- [ ] Make `preview --refresh-sources` refresh path Skill Source hashes in memory only.
- [ ] Make `apply --refresh-sources` update active lockfiles.
- [ ] Make `apply --refresh-sources` materialize refreshed Managed Skill Content.
- [ ] Thread `SkillSourceResolutionPolicy` into lower-level resolution APIs without using CLI flag-shaped booleans.
- [ ] Verify preview without Source Refresh and `preview --refresh-sources` do not write persistent state.
- [ ] Add tests for changed Skill Source content and read-only preview behavior.

Validation:

```sh
cargo test --workspace source_refresh read_only_preview
```

### M5: Client Discovery Registry, Preview, and CLI Rendering

Goal: produce structured preview results once and render them through the CLI.

#### Task M5.1: Implement built-in Client Discovery Registry

- [ ] Add V1 default Clients and skill Client Discovery Location paths.
- [ ] Key Client Discovery Registry entries by `{configured_item_kind, client, install_level}` with only `configured_item_kind = "skill"` in V1; serialized forms may keep `resource_kind` and Persisted Scope Value until schema migration.
- [ ] Represent project and user Client Discovery Location paths.
- [ ] Represent confidence/provenance metadata for diagnostics.
- [ ] Resolve shared `.agents/skills/{name}` Client Discovery Location paths for Codex, Pi, OpenCode, and Cursor.
- [ ] Resolve Cline through `.cline/skills/{name}` and `~/.cline/skills/{name}` with experimental provenance.
- [ ] Do not model shared `.agents` support as a client-family interface; use multiple client entries that resolve to the same path.
- [ ] Add tests for every built-in Client Discovery Location.

Validation:

```sh
cargo test -p agentcfg-core discovery_registry
```

#### Task M5.2: Build Desired State from active Config Layers

- [ ] Combine Shared Project Config and User Project Config for project commands.
- [ ] Use only User Config for `--user` commands.
- [ ] Apply additive client selection across active Config Layers.
- [ ] Add repeatable CLI `--client` for `preview`, `apply`, `prune`, and `status`; carry it through workflow requests as a client filter.
- [ ] Treat omitted `--client` as all Clients selected by active Config Layers.
- [ ] Validate that each requested `--client` is both a known V1 Client and selected by the active config; when config uses `clients = "all"`, allow any supported V1 Client. Do not let CLI flags add Clients outside the configured selection.
- [ ] Convert Skill Selection output into structured Desired State entries for Installed Artifacts before previewing apply changes.
- [ ] Include configured item kind, Client Discovery Location path, symlink mode, Managed Skill Content path, Discovery Name, installed hash, Skill Source/Config Layer provenance, and Discovery Requirements keyed by Config Layer, Client, and Install Level.
- [ ] Expose Desired State construction as a lower-level core API that `preview`, `apply`, `status`, and `prune` can share.
- [ ] Add tests for project layering, user-only mode, and shared Client Discovery Location Discovery Requirements.

Validation:

```sh
cargo test -p agentcfg-core desired_state
```

#### Task M5.3: Generate structured preview entries

- [ ] Re-evaluate whether `workflow/types.rs` should split (for example `workflow/types/init.rs` vs preview/apply request types) once preview and apply result fields are defined; avoid a single growing DTO module before adding M5 fields.
- [ ] Detect Discovery Name Collisions per Client Discovery Location path after Client Discovery Registry resolution.
- [ ] Merge Discovery Requirements only when selected entries refer to the same locked Source Skill Name and installed hash.
- [ ] Include Config Layer/Skill Source context in collision diagnostics.
- [ ] Generate lockfile create/update entries.
- [ ] Generate Installed Artifact create/update entries.
- [ ] Generate Discovery Requirement addition entries.
- [ ] Generate Stale Discovery Requirement and Stale Installed Artifact entries for reporting only.
- [ ] Keep preview operation records structured and free of terminal formatting.
- [ ] Keep preview operation records configured-item-kind aware but skill-first; do not add generic Configured Item manager interfaces.
- [ ] Expose the operation builder as a lower-level core API that consumes Desired State entries and current lock/manifest state.
- [ ] Add tests for create, update, no-op, and stale reporting previews.

Validation:

```sh
cargo test -p agentcfg-core preview_operations
```

#### Task M5.4: Render `preview` output in the CLI

- [ ] Render structured preview entries as human-readable terminal output.
- [ ] Include Discovery Name preparation and uncertain Client Discovery Location warnings.
- [ ] Render structured empty–Skill Source discovery warnings from core (see M2.2).
- [ ] Ensure `agentcfg preview` performs no persistent writes.
- [ ] Add CLI snapshot or assertion tests for representative preview output.

Validation:

```sh
cargo test --workspace preview_render
```

### M6: Apply, Manifest, and Prune

Goal: safely mutate only manifest-owned Installed Artifacts.

#### Task M6.1: Define manifest models and JSON persistence

- [ ] Model manifest records with configured item kind, Skill Source id, Source Skill Name, Discovery Name, Client Discovery Location path, symlink kind, installed hash, Discovery Requirements, created-by marker, Skill Source acquisition mode, and symlink mode. Serialized fields may keep names such as `resource_kind`, `target_path`, `target_kind`, `target_mode`, and `consumers` until schema migration.
- [ ] Read and write project and user manifests.
- [ ] Preserve structured Discovery Requirements by Config Layer, Client, and Install Level.
- [ ] Add round-trip and ordering tests.

Validation:

```sh
cargo test -p agentcfg-core manifest
```

#### Task M6.2: Apply Installed Artifact creates and updates

- [ ] Expose user-level Managed State paths through `WorkflowContext` (delegate to `UserDirs::state_home()` and `ManagedStatePaths::for_user`) so apply/status/prune do not re-read environment variables ad hoc.
- [ ] Create Client Discovery Location symlinks to Managed Skill Content.
- [ ] Update manifest-owned symlinks only when the current symlink target matches the manifest `target_path`.
- [ ] Refuse to overwrite Unmanaged Artifacts or Unexpected Symlink Targets.
- [ ] Add required Discovery Requirements to manifest records.
- [ ] Warn when Stale Installed Artifacts remain after apply.
- [ ] Expose apply as a lower-level core API that consumes structured preview operation entries; keep terminal warnings in the CLI renderer.
- [ ] Add tests for create, safe update, Unmanaged Artifact conflict, and Unexpected Symlink Target refusal.

Validation:

```sh
cargo test -p agentcfg-core apply_install
```

#### Task M6.3: Detect Stale Discovery Requirements and Stale Installed Artifacts

- [ ] Compare manifest Discovery Requirements against Desired State.
- [ ] Mark removed Config Layer/Client pairs as Stale Discovery Requirements.
- [ ] Mark Installed Artifacts with no remaining Discovery Requirements as Stale Installed Artifacts.
- [ ] Detect unused Managed Skill Content as Managed State leftovers.
- [ ] Expose stale-state detection as a lower-level core API shared by preview reporting and prune.
- [ ] Add tests for shared `.agents/skills` Discovery Requirements across Codex, Pi, OpenCode, and Cursor.

Validation:

```sh
cargo test -p agentcfg-core stale_state shared_discovery_requirements
```

#### Task M6.4: Implement prune safety engine

- [ ] Remove Stale Discovery Requirements.
- [ ] Remove Installed Artifacts only when no Discovery Requirements remain.
- [ ] Refuse to prune Unexpected Symlink Targets.
- [ ] Never delete Unmanaged Artifacts that are real files.
- [ ] Delete directories only if empty and manifest-owned.
- [ ] Expose prune apply as a lower-level core API that consumes stale-state records.
- [ ] Add tests for each safety invariant.

Validation:

```sh
cargo test -p agentcfg-core prune
```

### M7: Status and Doctor

Goal: expose local consistency and environment diagnostics without duplicating operation-generation logic.

#### Task M7.1: Implement structured status checks

- [ ] Report Installed Artifacts by Client.
- [ ] Report Broken Symlinks and Unexpected Symlink Targets.
- [ ] Report missing Managed Skill Content.
- [ ] Report Stale Installed Artifacts and unused Managed Skill Content.
- [ ] Report config/lock mismatch.
- [ ] Report Unmanaged Artifacts as informational unless they conflict.
- [ ] Expose status checks as structured core results so CLI rendering stays separate.
- [ ] Add tests using temporary manifests and Client Discovery Location directories.

Validation:

```sh
cargo test -p agentcfg-core status
```

#### Task M7.2: Render `status` in the CLI

- [ ] Render structured status results.
- [ ] Use script-friendly output conventions where practical.
- [ ] Map inconsistent state to the intended exit code.
- [ ] Add CLI output tests for consistent and inconsistent states.

Validation:

```sh
cargo test --workspace status_render
```

#### Task M7.3: Implement structured doctor checks

- [ ] Check git availability.
- [ ] Check Project Root detection.
- [ ] Check supported Clients and Client Discovery Location confidence.
- [ ] Check path writability.
- [ ] Check config schema validity.
- [ ] Check Unmanaged Artifacts only when they block previewed Client Discovery Location paths.
- [ ] Keep optional network/Skill Source checks isolated so local doctor remains deterministic in tests.
- [ ] Optionally report configured path Skill Sources that exist but discover zero skills (informational/warning; not required for M2.1).
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

### M8: Git Skill Sources

Goal: add git Skill Sources by reusing the path Skill Source discovery, materialization, hashing, lockfile, and preview/apply operation pipeline.

#### Task M8.1: Model git Skill Source config and validation

- [ ] Extend Skill Source config parsing for `type = "git"`.
- [ ] Validate required git fields.
- [ ] Preserve requested ref separately from resolved commit.
- [ ] Add parsing and validation tests.

Validation:

```sh
cargo test -p agentcfg-core git_config
```

#### Task M8.2: Resolve git Skill Sources into local checkouts

- [ ] Clone or fetch git Skill Sources into a Skill Source resolution checkout under Managed State.
- [ ] Resolve floating refs to concrete commits.
- [ ] Support pinned commit refs without treating them as floating.
- [ ] Keep network/git command execution behind an injectable boundary for tests.
- [ ] Add tests using local fixture git Skill Sources.

Validation:

```sh
cargo test -p agentcfg-core git_resolution
```

#### Task M8.3: Discover and materialize git skills through the existing pipeline

- [ ] Run path Skill Source discovery against the resolved git checkout root using the same `discovery_depth` and traversal rules as path Skill Sources.
- [ ] Reuse Skill Source-local Skill Group resolution.
- [ ] Reuse safe materialization and hashing.
- [ ] Reuse Discovery Name preparation behavior.
- [ ] Add tests proving path and git Skill Sources produce equivalent Managed Skill Content for equivalent content.

Validation:

```sh
cargo test -p agentcfg-core git_materialization
```

#### Task M8.4: Implement locked git apply behavior

- [ ] Reuse locked Managed Skill Content for git Skill Sources during plain apply.
- [ ] Recreate missing Managed Skill Content from locked commits when available.
- [ ] Fail clearly when a locked commit cannot be fetched or restored.
- [ ] Add tests using local fixture git Skill Sources.

Validation:

```sh
cargo test -p agentcfg-core locked_git_apply
```

#### Task M8.5: Implement git Source Refresh behavior

- [ ] Make `preview --refresh-sources` detect moved floating refs without persistent writes.
- [ ] Make `apply --refresh-sources` update lockfiles to resolved commits.
- [ ] Make `apply --refresh-sources` materialize refreshed Managed Skill Content.
- [ ] Add tests for floating ref movement and pinned commit no-op.

Validation:

```sh
cargo test -p agentcfg-core git_source_refresh
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
- [ ] `agentcfg preview` performs no persistent writes.
- [ ] `agentcfg apply` installs a path Skill Source skill from Locked Desired State in Managed State.
- [ ] `agentcfg apply --refresh-sources` imports changed path Skill Source content.
- [ ] `agentcfg prune` removes only manifest-owned Stale Installed Artifacts and Stale Discovery Requirements.
- [ ] Discovery Name Collision behavior is covered.
- [ ] Internal symlink materialization and external symlink rejection are covered.
- [ ] Shared `.agents/skills` Discovery Requirements across Codex/Pi/OpenCode/Cursor are covered.
- [ ] Cline native `.cline/skills` Client Discovery Location behavior is covered with experimental provenance.
- [ ] Git Skill Source apply and Source Refresh behavior are covered by local fixture git Skill Sources.

## Open Planning Questions

- [X] Should git Skill Sources be included before or after the first path Skill Source apply milestone? YES.
- [ ] How much Skill Source provenance should be exposed in normal command output versus `doctor`?
- [X] Should V1 support both `skills/<name>/SKILL.md` and root-level `<name>/SKILL.md` Skill Source layouts? YES — bounded discovery to `discovery_depth` with nested-skill exclusion; see M2.1 and `design-v1.md` § Skill Source Discovery.
