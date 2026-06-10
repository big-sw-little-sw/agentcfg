# Version 1 Skills CLI and Library

**Status:** Draft
**Context(s):** Agent Configuration
**Date:** 2026-06-09

## Problem

Users need a repeatable way to make agent-facing Skills available to the Clients they use across Projects and User environments. The current shape is configuration-first, but V1 needs to become a working product without baking terminal UI assumptions into the domain logic.

V1 must support Skills only, while leaving a clean path for near-term configured item kinds such as subagent configuration and MCP configuration. It must also ship as a CLI now without making future Presentations, such as a TUI, interactive CLI, GUI, or editor integration, reimplement core workflow behavior.

## Solution

V1 will ship a Rust Cargo workspace with separate Core Crate and CLI Crate boundaries.

The Core Crate provides a presentation-agnostic Workflow API for SelectSkill, DeselectSkill, ConfigShow, Preview, Install, Prune, Status, and Doctor. Workflows return structured results, diagnostics, blockers, and progress events that any Presentation can render. The CLI Crate parses command-line input, renders terminal output, maps workflow results to exit codes, and delegates domain behavior to the Core Crate.

V1 supports Skill Configuration using Agent Configuration Files named `agentcfg.toml` and Agent Configuration Lockfiles named `agentcfg.lock`. Skill Sources may be GitHub shorthand such as `getsentry/dotagents`, full git URLs, local git repositories, or non-git local filesystem paths. All git-backed Skill Sources support Git Source Refs such as branches, tags, commits, labels that resolve to git refs, and other refs accepted by git. Supported Clients are Codex, Pi, OpenCode, Claude Code, Cline, and Cursor.

## User Stories

1. As a User, I want to declare Skills in an Agent Configuration File, so that the same Skills can be installed repeatably for supported Clients.
2. As a User, I want to select a Skill from GitHub shorthand, a full git URL, a local git repository, or a non-git local path and then install it explicitly, so that I can use both shared Skills and local development Skills without confusing configuration changes with installation.
3. As a User, I want to pin any git-backed Skill Source to a branch, tag, commit, label that resolves to a git ref, or other git ref, so that installation is repeatable from the intended source version.
4. As a User, I want to select one Skill or all enumerated Skills from a Skill Source, so that common source layouts are ergonomic.
5. As a User, I want to exclude specific Skills when selecting every enumerated Skill from a Skill Source, so that broad selections can omit Skills I do not want installed.
6. As a User, I want to apply Skill Aliases, so that configured item names can match how I want Skills to appear to Clients.
7. As a User, I want to choose Project Level or User Level installation, so that shared Project configuration and personal User configuration remain separate.
8. As a User, I want Install to produce repeatable Managed Skill Trees and Client Binding Artifacts from a PinnedConfig, so that a future run installs the same content unless I request Source Refresh.
9. As a User, I want Preview to show what Install would change without writing anything, so that I can inspect the effect of configuration changes before mutation.
10. As a User, I want Status to report whether each relevant Install Level's ObservedInstallation matches its LockfilePinnedConfig, so that manual drift is visible without hiding where it belongs.
11. As a User, I want Install to stop exposing deselected Skills and Prune to reclaim stale Managed State only when ownership can be derived, so that unmanaged files are not deleted.
12. As a User, I want Doctor to report missing prerequisites, invalid configuration, unsupported Clients, and inaccessible Client Search Locations across relevant Install Levels, so that setup problems are actionable.
13. As a User, I want to deselect a Skill from configuration and then run Install, so that unneeded Skills stop being exposed to Clients through the same explicit materialization workflow as other configuration changes.
14. As a User, I want to inspect the Agent Configuration that applies at a selected level, so that I can understand selected Skills, aliases, exclusions, and client selections without reading TOML directly.
15. As a developer building a Presentation, I want one Workflow API for all V1 workflows, so that CLI, TUI, interactive CLI, GUI, and editor integrations can share behavior.
16. As a developer adding future configured item kinds, I want shared resolution, lockfile, planning, installation, and ownership contracts, so that subagent configuration and MCP configuration do not fork the workflow model.

## Implementation Decisions

### Scope And Crate Boundaries

- V1 is skills-only. Skill Configuration is the only implemented configured item kind, but shared workflow concepts use Configured Item language where the behavior is not skill-specific.
- The Rust workspace has separate Core Crate and CLI Crate boundaries. The Core Crate is a library crate; the CLI Crate is a binary crate that depends on the Core Crate.
- The Core Crate owns ConfigDoc parsing, ConfigRequest normalization, Source Enumeration, Source Resolution, PinnedConfig creation, Agent Configuration Lockfile I/O, ObservedInstallation, InstallPlan construction, Derived Ownership checks, and execution of InstallPlan and Conservative Prune.
- The CLI Crate owns command parsing, terminal rendering, help text, process exit codes, and argument-to-workflow request mapping. It does not inspect filesystem state or mutate Managed Artifacts except through the Workflow API.
- Future configured item kinds should implement item-kind-specific source enumeration, source resolution, content preparation, and client adaptation behind the shared workflow, planning, and ownership contracts.

### Workflow API And CLI Contract

- The Workflow API exposes high-level operations for SelectSkill, DeselectSkill, ConfigShow, Preview, Install, Prune, Status, and Doctor. The V1 CLI presents SelectSkill and DeselectSkill as `agentcfg skills select` and `agentcfg skills deselect` to emphasize configuration selection rather than installation.
- Each workflow accepts typed request data and returns a typed result with diagnostics, blockers, progress events, Suggested Actions, and workflow-specific result data suitable for any Presentation.
- The V1 CLI supports human text output by default and JSON output for every workflow through `--format text|json`, including configuration mutation and materialization workflows. JSON output is rendered from the same structured Workflow API results as human text output; it is not a separate reporting model.
- In human text mode, final command results are written to stdout and Progress Events are written to stderr. JSON output emits one final structured result object on stdout, writes no Progress Events in V1, and keeps stdout machine-readable.
- JSON output must preserve diagnostics, blockers, planned changes, completed changes, skipped artifacts, Config Layer provenance, Install Level partitioning, exit-relevant status, and Suggested Actions. Process exit codes remain authoritative for success or failure in both formats.
- Human text rendering may turn Suggested Actions into prose; JSON output must keep them structured, such as command plus reason. Suggested Actions may name mutating commands, but they are follow-up guidance only and are never executed automatically.
- CLI exit behavior is workflow-specific. `status` exits zero only when all reported configured levels are consistent or empty; missing lockfiles, missing pins, installation drift, invalid config, unreadable required paths, and Unmanaged Conflicts exit nonzero. `doctor` exits zero only when all enabled checks pass or reported levels are empty; failed checks exit nonzero. `preview` exits zero when it successfully computes a plan, even when the plan contains changes, and exits nonzero for blockers or failures. `install` and `prune` exit zero when the requested mutation completes or is already a no-op, and exit nonzero for blockers or failures. Config show commands exit zero when they render valid configuration or empty levels, and exit nonzero for invalid or unreadable configuration. Config mutation commands exit zero when the requested change succeeds or is already satisfied, and exit nonzero for invalid input, validation blockers, or write failures.
- V1 Diagnostics do not need severity classification. Their structured shape must preserve stable codes, contextual data, and optional Suggested Actions so severity can be added in a later version without changing workflow behavior. The V1 spec does not enumerate the full diagnostic code catalog; codes are introduced with implementation and must be documented and test-covered once introduced.
- Progress Events are emitted for major workflow phases, not fine-grained tracing. V1 phases include resolving external sources, planning, writing Agent Configuration Files and Agent Configuration Lockfiles, preparing Managed State, writing Client Binding Artifacts, applying Conservative Prune, and running Doctor checks.

### Locations, Layers, And Levels

- Agent Configuration Files are named `agentcfg.toml`; Agent Configuration Lockfiles are named `agentcfg.lock`.
- Project Root resolves to the top-level git repository directory, or the current working directory when no git repository is found.
- Shared Project Config lives at `agentcfg.toml` under the Project Root. Its Agent Configuration Lockfile lives at `agentcfg.lock` under the Project Root.
- User Project Config lives at `.agentcfg/agentcfg.toml` under the Project Root. Its Agent Configuration Lockfile lives at `.agentcfg/agentcfg.lock` under the Project Root.
- User Config lives at `$XDG_CONFIG_HOME/agentcfg/agentcfg.toml`, or `$HOME/.config/agentcfg/agentcfg.toml` when `XDG_CONFIG_HOME` is not set. Its Agent Configuration Lockfile follows the same directory and is named `agentcfg.lock`.
- Project Managed State is content-addressed under `.agentcfg/state` at the Project Root and is preferred for Project Level workflows.
- Project Managed State is shared by all active Project Config Layers. Shared Project Config and User Project Config retain separate Agent Configuration Files and Agent Configuration Lockfiles, but Project Level Install prepares their Managed Artifacts under the same Project Managed State root.
- User Managed State is content-addressed under `$XDG_STATE_HOME/agentcfg`, or `$HOME/.local/state/agentcfg` when `XDG_STATE_HOME` is not set, and is used for User Level workflows.
- Project Level workflows read active Config Layers from Shared Project Config followed by User Project Config. User Level workflows read User Config only.
- V1 Active Config Layers are additive. User Project Config can add Skill Configuration alongside Shared Project Config, but it cannot override, subtract from, or mutate Skill Configuration from Shared Project Config.
- V1 CLI uses `--config-layer` for commands that mutate or narrow inspection to a specific Agent Configuration File and `--level` for commands that materialize or inspect an Install Level. Help text must describe `--config-layer` as the configuration file to edit or inspect and `--level` as the installation level to inspect or materialize. Install Level help must make clear that Project Level reads both Shared Project Config and User Project Config.
- `agentcfg preview`, `agentcfg install`, and `agentcfg prune` default to Project Level when `--level` is omitted. User Level preview, installation, and pruning require explicit `--level user`.

### Configuration Inspection And Mutation

- Config mutation workflows update the selected Config Layer only. They do not run Install, Prune, Source Refresh, or any Managed Artifact mutation.
- V1 exposes general configuration inspection as `agentcfg config show`. `config show` defaults to Project Level, reports all active Config Layers for the selected level partitioned by Config Layer, supports `--config-layer` narrowing, requires `--level user` for User Config inspection, and does not mutate files or Managed Artifacts.
- `config show` reports authored configuration plus cheap local normalization only. It does not read Agent Configuration Lockfiles, perform Source Enumeration, perform Source Resolution, use git, or use network access. It validates schema and local shape, such as malformed TOML, invalid client names, duplicate fields, and invalid or mixed selection syntax, but it does not validate source-dependent facts such as whether included skills, excluded skills, or alias keys exist in a Skill Source.
- Because V1 implements Skill Configuration only, `config show` renders Skill Configuration, Default Client Selection, entry-level clients, Skill Sources, Skill Selection, Excluded Skills, and Skill Aliases without expanding all-skill selections from source contents.
- Project Level config mutation workflows default to User Project Config. Writing Shared Project Config requires explicit `--config-layer shared-project`.
- SelectSkill is exposed as `agentcfg skills select`. It does not accept client selection; new entries inherit the Config Layer's Default Client Selection. `skills select` fails before writing when the new or updated entry would not have a final client selection, and the diagnostic suggests `agentcfg clients set <client>...` before selecting Skills.
- When selecting an explicit Included Skill, SelectSkill appends to an existing compatible Skill Configuration Entry instead of creating a duplicate entry. Compatibility requires the same Skill Source identity, Git Source Ref when present, final client selection, and no conflicting aliases or exclusions. `skills select` output reports the selected entry's inherited or explicit Clients, mentions `agentcfg skills clients ...` for client changes, and tells the User to run `agentcfg install` to materialize changes.
- DeselectSkill is exposed as `agentcfg skills deselect`. Deselecting from User Project Config does not hide Skill Configuration still contributed by Shared Project Config. `skills deselect` output tells the User to run `agentcfg install` to materialize binding changes and run `agentcfg prune` when stale Managed State should be removed.
- V1 includes Default Client Selection workflows: `agentcfg clients show`, `agentcfg clients set <client>...`, `agentcfg clients add <client>...`, and `agentcfg clients remove <client>...`. Mutation output reports which entries inherit the changed default and tells the User to run Install to materialize changes.
- `clients show` is read-only configuration inspection: it defaults to Project Level, reports all active Config Layers for the selected level partitioned by Config Layer, supports `--config-layer` to narrow the report, and requires `--level user` to inspect User Config.
- V1 includes Skill Configuration Entry client workflows: `agentcfg skills clients show`, `agentcfg skills clients set`, `agentcfg skills clients add`, `agentcfg skills clients remove`, and `agentcfg skills clients inherit`.
- Skill Configuration Entry client mutations select an entry with explicit flags such as `--source`, `--skill`, and disambiguators such as `--ref` or `--path` when needed. `--skill` matches Source Skill Name, not Skill Alias. Selectors that match zero or multiple entries fail and report enough matching entry detail for the User to disambiguate.
- For Skill Configuration Entry client mutations, `set` replaces entry-level clients, `add` and `remove` create explicit entry-level clients when the entry inherited the Default Client Selection, and `inherit` removes entry-level clients so the entry inherits the Default Client Selection again. Mutation output tells the User to run Install to materialize changes.
- `skills clients show` is read-only configuration inspection and follows the same level and Config Layer reporting defaults as `clients show`.

### Source Model, Lockfiles, And Refresh

- Skill Sources support non-git local filesystem paths, local git repositories, GitHub shorthand, and full git URLs.
- All git-backed Skill Sources support Git Source Refs, including branches, tags, commits, labels that resolve to git refs, and other refs accepted by git. Source Resolution records repeatable resolved refs and content identities in the PinnedConfig.
- Local git repositories are resolved like other git-backed Skill Sources after the repository path is normalized.
- Non-git local filesystem Skill Sources resolve to Source Kind `local-filesystem`, a normalized source path, and an algorithm-qualified Content Hash. The Content Hash is the authoritative content identity for repeatable installation and Managed Skill Tree equality.
- The normalized source path for a non-git local filesystem Skill Source is provenance and the Source Refresh locator. It is used for diagnostics and re-reading the source during Source Refresh, but it is not proof of content and must never replace Content Hash verification.
- Source Resolution computes non-git local filesystem Content Hash values from a deterministic tree representation: relative paths, file bytes, and intentionally preserved mode bits. It excludes mtime, ctime, inode, device, owner, group, uid/gid, and host-specific metadata. PinnedConfig must not persist mtime or other mutable filesystem metadata for non-git local filesystem Skill Sources.
- Agent Configuration Lockfile writes follow Config Layer ownership. During Project Level Install, missing pins for entries declared by Shared Project Config are written to the Shared Project Config lockfile, and missing pins for entries declared by User Project Config are written to the User Project Config lockfile. If the owning lockfile cannot be written, Install blocks with a diagnostic instead of pinning the entry into another Config Layer's lockfile.
- Source Refresh is an explicit option on Preview and Install for refreshing existing pins. Missing pins are resolved by Preview and Install even when Source Refresh is not requested.
- Install without Source Refresh installs locked content. If the required Managed Skill Tree is missing, Install may rebuild it from the normalized local source only when the current Content Hash matches the LockfilePinnedConfig; otherwise it blocks with a Diagnostic containing Source Kind, normalized path, expected hash, and observed hash.
- Source Refresh for a non-git local filesystem Skill Source re-hashes the normalized source path and produces a refreshed PlannedPinnedConfig when the Content Hash changes. Preview keeps that refreshed plan in memory; Install persists the refreshed Agent Configuration Lockfile before artifact writes. Missing, inaccessible, or changed sources required for missing-pin resolution or Source Refresh are blockers with structured Diagnostics and never cause writes back to item sources.

### Skill Enumeration, Selection, And Client Resolution

- Source Enumeration for Skills lists Source Skill Names before Skill Selection is finalized.
- V1 Source Enumeration accepts either a single root `SKILL.md` or child directories containing `SKILL.md`. A Skill Source containing both shapes is ambiguous and fails validation before Skill Selection.
- Source Skill Name comes from the `name` declared in `SKILL.md` frontmatter for both root and child-directory skill shapes. Missing, invalid, or duplicate frontmatter names fail validation during Source Enumeration; source directory names are layout and provenance, not Source Skill Names.
- Each Skill Configuration Entry has exactly one Skill Selection mode: explicit Included Skills or all enumerated Skills. V1 persisted configuration represents explicit selection as `include = ["source-skill-name"]` and all-skill selection as `include = "all"`; `include = "*"` and mixed selection modes are invalid.
- Skill Selection supports Excluded Skills when a Skill Configuration Entry selects every enumerated Skill from a Skill Source. Exclusions are Source Skill Names, apply only to the declaring entry, remove matching Source Skill Names before aliases produce Configured Item Names, and fail validation when they do not match an enumerated Source Skill Name. Excluded Skills are invalid on entries that explicitly list Included Skills.
- V1 Skill Aliases are explicit per-Source-Skill mappings and may be used with explicit or all-skill selection. Alias keys must match Source Skill Names that remain selected after Excluded Skills are removed; aliases for unselected or excluded Source Skill Names fail validation. Pattern, prefix, or default alias rules are out of scope.
- Client selection belongs to the Skill Configuration Entry that selects the Skill. A Skill exposed to more Clients is represented by updating that entry's client selection, and resolution derives one Client Binding per selected Client.
- A Config Layer may declare a Default Client Selection used by configured item entries that do not declare entry-level clients. Entry-level clients replace the Default Client Selection for that entry. ConfigRequest normalization must produce an explicit final client selection for every entry, and validation fails before PinnedConfig creation when an entry has neither entry-level clients nor a Default Client Selection. Unsupported configured item and Client combinations fail validation before PinnedConfig creation.
- Duplicate Configured Item Names across the Active Config Layers after Skill Alias resolution are a validation error surfaced to the User, even when the duplicate entries target different Clients. Workflows must not produce a PinnedConfig or InstallPlan when duplicates remain unresolved.

### Planning, Install, Status, Doctor, And Prune

- Preview is scoped to one Install Level, uses the same level selection as Install, resolves missing lockfile pins in memory because Install would create them before artifact writes, reports planned lockfile changes grouped by owning Config Layer, keeps the PlannedPinnedConfig in memory, and writes nothing.
- Preview cannot compute a complete install plan when a source required for missing-pin resolution is unavailable; that condition is a blocker.
- Install resolves active Agent Configuration Files, creates missing lockfile pins required by newly selected configured items, persists Agent Configuration Lockfile changes, reconciles Client Binding Artifacts for the active Install Level, and installs required Managed Artifacts.
- Install removes stale Client Binding Artifacts for the active Install Level only when ownership can be derived. Install does not run Prune and does not remove stale Managed State artifacts such as unused Managed Skill Trees; stale Managed State cleanup remains explicit Prune behavior.
- Install reports lockfile updates grouped by owning Config Layer and Managed Artifact changes grouped by Install Level and Client. Install never mutates Client Binding Artifacts or Managed State for an inactive Install Level; cross-level cleanup requires running Install or Prune for that level.
- Installation planning compares PlannedPinnedConfig with ObservedInstallation and produces an InstallPlan. Unmanaged Conflicts block mutation instead of being overwritten.
- Status and Doctor are read-only assessment workflows. In V1, `agentcfg status` and `agentcfg doctor` default to all relevant Install Levels and support `--level project` and `--level user` to narrow the report. Results are sectioned by Install Level; Project Level results preserve which findings come from Shared Project Config versus User Project Config when that distinction matters. An Install Level with no Agent Configuration is reported as a normal empty state, not as a warning or blocker.
- `status` compares LockfilePinnedConfig with ObservedInstallation and does not perform Source Enumeration, Source Resolution, git access, network access, or source reachability checks. When a required Agent Configuration Lockfile is missing, Status reports that expected state cannot be computed and suggests `agentcfg install` to recreate pins; it must not report selected items as installed or not installed. When an existing lockfile lacks a pin for a selected item, Status reports install required. When a lockfile pin exists but the required Managed Artifact or Client Binding Artifact is absent, Status reports installation drift.
- `doctor` is local by default. It validates environment capabilities, supported Clients, Config Layer readability, lockfile readability, Managed State accessibility, Client Search Location accessibility, and local source path availability. Doctor reports Managed State that exists without readable corresponding Agent Configuration or lockfile context as a local diagnostic and points Users to Status or Prune; it does not classify that state as drift or remove it. Remote git or network source reachability checks run only when the User passes `--check-sources`.
- Prune uses Conservative Prune to remove stale Managed State artifacts for the active Install Level, such as unused Managed Skill Trees, only when ownership can be derived from lockfiles, deterministic managed paths, Client Adapter Catalog rules, binding artifact shape, or symlink targets. Prune may remove stale Managed State when a lockfile is missing only if ownership can still be derived from non-lockfile evidence such as deterministic managed paths or artifact shape; missing lockfiles reduce ownership evidence but do not by themselves permit or forbid cleanup.
- Prune reports removed stale Managed State artifacts by Install Level, removed stale Client Binding Artifacts by Client, and skipped artifacts with explicit safety reasons such as ownership not derived, Unmanaged Conflict, or inactive Install Level. Prune never mutates inactive Install Level artifacts, never removes Unmanaged Conflicts, and is not required for Install to stop exposing deselected Skills through Client Binding Artifacts.

### Managed Skill Trees And Client Adapters

- V1 prepares one Managed Skill Tree per Configured Item Name in the active Install Level's Managed State. The prepared tree's final directory name and `SKILL.md` frontmatter `name` must match the Configured Item Name when a Skill Alias is applied or when a target Client requires, validates, or derives behavior from name-to-directory identity.
- Rewriting `SKILL.md` frontmatter `name` for a Skill Alias is item-wide Managed Skill Tree preparation, not Client Adaptation. V1 preserves the `SKILL.md` filename and supporting files, never writes prepared changes back to Skill Sources, and does not synthesize optional client metadata files unless explicitly configured.
- V1 Client Adaptation for Skills is limited to adapter-owned Client Binding Artifact shape and Client Search Location layout. No V1 Client requires an alternate skill filename or a different required metadata file for the common Agent Skill Format path.
- The Client Adapter Catalog supports Codex, Pi, OpenCode, Claude Code, Cline, and Cursor in V1.
- Client Binding Artifact shape is adapter-owned. V1 may use symlinks where a Client supports loading Skills from a directory, but the safety model must not assume all future adapters use symlinks.
- Client Adaptation is adapter-owned and separate from Skill Alias. Skill Alias changes the item-wide Configured Item Name; Client Adaptation changes only what a Client needs to load the item.

## Testing Strategy

### Workflow Surfaces And CLI Contract

- Test the Core Crate through public Workflow API entry points for SelectSkill, DeselectSkill, ConfigShow, Preview, Install, Prune, Status, and Doctor.
- Test CLI behavior at the command boundary: arguments map to workflow requests, `--config-layer` is accepted for configuration mutation and configuration show narrowing, `--level` is accepted for materialization, assessment, and level-scoped configuration show commands, help text explains the distinction, terminal output renders key diagnostics, and exit codes reflect workflow-specific success, action-required assessment findings, blockers, invalid input, and execution failures.
- Test `--format text|json` for every workflow, including configuration mutation and materialization workflows, with `text` as the default. In text mode, final command results are written to stdout and Progress Events to stderr. JSON output is an alternate rendering of the same structured Workflow API results, emits one final structured result object on stdout, writes no Progress Events in V1, keeps stdout machine-readable, preserves diagnostics, blockers, planned changes, completed changes, skipped artifacts, Config Layer provenance, Install Level partitioning, exit-relevant status, and Suggested Actions, and keeps process exit codes authoritative for success or failure.
- Test introduced Diagnostic codes as stable structured outputs with contextual data, without requiring the V1 spec to enumerate the complete code catalog upfront.
- Avoid duplicating Core Crate behavior in CLI tests. CLI tests should use either the real Core Crate with small fixtures or a narrow workflow harness where command rendering is the behavior under test.

### Config Documents, Layers, And Inspection

- Test ConfigDoc parsing and writing with `agentcfg.toml`, including Config Layer values, Default Client Selection, entry-level client selection, Skill Sources, Skill Selection, Excluded Skills, and Skill Aliases.
- Test ConfigShow through `agentcfg config show`: it defaults to Project Level, reports all active Config Layers for the selected level partitioned by Config Layer, supports `--config-layer` narrowing, requires `--level user` for User Config inspection, renders V1 Skill Configuration fields, validates schema and local shape, does not validate source-dependent facts, does not read Agent Configuration Lockfiles, does not perform Source Enumeration or Source Resolution, does not expand all-skill selections from source contents, and does not mutate Agent Configuration Files, lockfiles, Managed State, or Client Search Locations.
- Test additive Active Config Layer semantics: User Project Config contributions are included alongside Shared Project Config, but exclusions, aliases, client choices, and removals in one Config Layer do not alter another Config Layer's Skill Configuration.

### Config Mutation And Client Workflows

- Test SelectSkill and DeselectSkill as config mutation workflows, with V1 CLI coverage for `agentcfg skills select` and `agentcfg skills deselect`: they update the selected Config Layer, do not mutate Managed Artifacts, and report the follow-up Install or Prune step. Project Level SelectSkill and Project Level DeselectSkill default to User Project Config and write Shared Project Config only when explicitly selected; deselecting from User Project Config does not hide Skill Configuration still contributed by Shared Project Config.
- Test SelectSkill client behavior: SelectSkill does not accept client selection, new entries inherit the Default Client Selection, SelectSkill fails before writing when an entry would have no final client selection, and SelectSkill appends explicit Included Skills to an existing compatible Skill Configuration Entry while creating a distinct entry only when compatibility checks fail.
- Test Default Client Selection workflows for show, set, add, and remove. Mutation workflows default Project Level writes to User Project Config, support explicit Shared Project Config mutation, report inherited entries with changed future Client Bindings, leave explicit entry-level clients unchanged, do not mutate Managed Artifacts, and fail when removing the last default Client while entries depend on the default. Show defaults to Project Level, reports all active Config Layers for the selected level partitioned by Config Layer, supports `--config-layer` narrowing, and requires `--level user` for User Config inspection.
- Test client selection as part of one Skill Configuration Entry: one selected Skill with multiple Clients derives multiple Client Bindings. Entry-level clients replace the Config Layer's Default Client Selection, unsupported Skill and Client combinations fail validation, and entries with no final client selection fail validation.
- Test Skill Configuration Entry client workflows for show, set, add, remove, and inherit, including selector matching by Source Skill Name, ambiguity failures, creating explicit entry-level clients from inherited defaults, returning to inheritance, no Managed Artifact mutation, follow-up Install messaging for mutations, and show reporting partitioned by Config Layer for the selected/default Install Level.

### Sources, Pinning, And Selection Validation

- Test Agent Configuration Lockfile read/write with `agentcfg.lock`, including repeatable source refs, Git Source Refs, content identities, Configured Item Names, and Client Bindings. Plain Install creates missing pins for newly selected configured items before artifact writes, writes each pin to the lockfile owned by the declaring Config Layer, and blocks instead of writing a pin into another Config Layer's lockfile.
- Test Skill Source Enumeration and Source Resolution for non-git local paths, local git repositories, GitHub shorthand, and full git URLs. Source Enumeration accepts root-only and child-directory-only Skill Sources, rejects mixed root-and-child Skill Source shapes as ambiguous, requires valid and unique `SKILL.md` frontmatter names as Source Skill Names, and uses filesystem fixtures for local sources and local git repositories. Mock git/network boundaries.
- Test non-git local filesystem pinning with deterministic Content Hash fixtures, normalized path diagnostics, no persisted mutable filesystem metadata, and changed-source blocking when Install runs without Source Refresh.
- Test Skill Selection modes as mutually exclusive: each Skill Configuration Entry selects explicit Included Skills with `include = ["source-skill-name"]` or all enumerated Skills with `include = "all"`, while `include = "*"`, mixed selection modes, and missing selection modes fail validation.
- Test Excluded Skills as entry-local Source Skill Names that are valid only with all-skill selection, invalid with explicit Included Skills, and invalid when they do not match enumerated Source Skill Names.
- Test Skill Aliases as explicit per-Source-Skill mappings with explicit and all-skill selection. Aliases for unselected or excluded Source Skill Names fail validation, and pattern, prefix, or default alias rules are rejected in V1 configuration.
- Test duplicate Configured Item Names across Active Config Layers after Skill Alias resolution as validation errors before PinnedConfig or InstallPlan creation, including duplicates whose client selections do not overlap.

### Materialization, Planning, And Prune Safety

- Test Preview, Install, and Prune default to Project Level and require explicit `--level user` for User Level. Test Preview and Install scope consistently by Install Level: Preview must show the same plan Install would execute for the selected Install Level, resolve missing pins in memory, report planned lockfile changes grouped by owning Config Layer, treat missing-pin source access failures as blockers, must not aggregate Project Level and User Level plans, and must not write refreshed pins. Install must persist missing or refreshed lockfile data before artifact writes.
- Test Install reconciliation of Client Binding Artifacts: required bindings are created or repaired, stale binding artifacts for the active Install Level are removed only when ownership can be derived, inactive Install Level artifacts are not mutated, output reports lockfile updates by owning Config Layer and Managed Artifact changes by Install Level and Client, Unmanaged Conflicts block mutation, Install does not run Prune, and stale Managed Skill Trees are left for Prune.
- Test Managed Skill Tree preparation for aliases: the prepared directory and `SKILL.md` frontmatter `name` match the Configured Item Name, supporting files are preserved, source files are not modified, and Project Level entries from Shared Project Config and User Project Config prepare into the same Project Managed State root.
- Test installation planning behavior from PlannedPinnedConfig and ObservedInstallation, including missing artifacts, stale Client Binding Artifacts, stale Managed State artifacts, broken symlinks, unexpected binding targets, and Unmanaged Conflicts.
- Test Derived Ownership and Conservative Prune as safety-critical core logic. Conservative Prune tests focus on stale Managed State artifacts such as unused Managed Skill Trees, verify cleanup with missing lockfiles only when ownership is derived from non-lockfile evidence, verify active Install Level isolation, report removals grouped by Install Level and Client, report skipped artifacts with explicit safety reasons, and should be behavioral and table-driven where possible.

### Assessment Workflows

- Test Status and Doctor as read-only assessment workflows: they default to all relevant Install Levels, support narrowing to Project Level or User Level, keep results partitioned by Install Level, preserve relevant Config Layer provenance in Project Level findings, report levels with no Agent Configuration as normal empty states, and do not mutate Agent Configuration Files, lockfiles, Managed State, or Client Search Locations.
- Test Status as lockfile-backed assessment: it compares LockfilePinnedConfig with ObservedInstallation, does not perform source checks, and distinguishes missing lockfiles, missing pins, and pinned-but-absent artifacts without reporting missing lockfiles as installed or not installed.
- Test Doctor as local capability assessment: it does not perform remote git or network source reachability checks by default, `--check-sources` enables those checks, and Managed State without readable corresponding Agent Configuration or lockfile context is reported as a diagnostic without being classified as drift or removed.

### Client Adapter Contracts

- Test Client Adapter Catalog contracts for each V1 Client: Codex, Pi, OpenCode, Claude Code, Cline, and Cursor.
- Test Client Adaptation separately from Skill Alias to prevent client-specific layout handling from changing item-wide names.

## Out of Scope

- Configured item kinds other than Skills, including subagent configuration and MCP configuration.
- VS Code Client support.
- TUI, interactive CLI, GUI, editor integrations, and other non-CLI Presentations.
- Bidirectional sync from Client Search Locations or Managed State back into Agent Configuration Files.
- Writing changes back to Skill Sources.
- Unsafe overwrite, force install, or force prune behavior for Unmanaged Conflicts.
- Remote registries or hosted package indexes beyond git and local filesystem sources.
- Authentication UX beyond reporting source access failures and using the user's existing git environment.
- Full schema migration framework for future config versions.

## Open Questions

None.
