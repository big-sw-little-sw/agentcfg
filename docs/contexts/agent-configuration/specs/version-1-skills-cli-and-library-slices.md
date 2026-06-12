# Version 1 Skills CLI and Library Slices

**Status:** Revised after project root resolution spec
**Spec:** [version-1-skills-cli-and-library.md](version-1-skills-cli-and-library.md)
**Date:** 2026-06-10

These slices implement the V1 Skills CLI and library as tracer bullets. Each slice should be independently demoable or verifiable, with tests included in the slice rather than deferred.

## 1. Config Show Tracer [done]

**Type:** AFK

### What to build

Create the minimal Rust Cargo workspace with Core Crate and CLI Crate boundaries, then implement a read-only Config Show workflow that can report an empty Project Level configuration through text and JSON CLI output.

### Acceptance criteria

- [ ] The workspace contains separate Core Crate and CLI Crate packages.
- [ ] The Core Crate exposes a presentation-agnostic Workflow API shape with structured workflow result data, Diagnostics, blockers, Suggested Actions, and Progress Events.
- [ ] The CLI Crate parses `agentcfg config show --format text|json` and delegates behavior to the Core Crate.
- [ ] Text output writes the final command result to stdout.
- [ ] JSON output writes one machine-readable final structured result object to stdout.
- [ ] The workflow reports an empty Project Level configuration as a normal state.
- [ ] Exit codes match the Config Show contract for success and invalid input.

### Testing scope

- Core workflow tests for empty Project Level Config Show.
- CLI boundary tests for argument mapping, text output, JSON output, and exit codes.
- Do not test full Skill Configuration schema yet.

### Blocked by

None - can start immediately.

## 2. Config Layers And Default Clients [done]

**Type:** AFK

### What to build

Make Config Layer and Install Level location resolution real, then add Default Client Selection inspection and mutation workflows for Shared Project Config, User Project Config, and User Config.

### Acceptance criteria

- [ ] Shared Project Config, User Project Config, and User Config paths are resolved according to the spec.
- [ ] Project Level reads active Config Layers in Shared Project Config then User Project Config order.
- [ ] User Level reads User Config only.
- [ ] User Config path resolution blocks when neither `XDG_CONFIG_HOME` nor `HOME` is set.
- [ ] Known V1 Client names are available for ConfigDoc parsing and config mutation validation without requiring Client Search Locations or Client Binding Artifact shapes.
- [ ] `agentcfg clients show` reports Default Client Selection partitioned by Config Layer.
- [ ] `agentcfg clients set <client>...`, `add`, and `remove` mutate only the selected Config Layer.
- [ ] Project Level client mutations default to User Project Config.
- [ ] Shared Project Config mutation requires `--config-layer shared-project`.
- [ ] User Config inspection and mutation require `--level user`.
- [ ] ConfigDoc client mutations preserve unrelated TOML content such as comments and key order.

### Testing scope

- Core tests for location resolution, active layer selection, Config Layer ownership, and known Client name validation.
- CLI tests for `clients show/set/add/remove`, `--level`, `--config-layer`, text/JSON output, and exit codes.
- Mutation tests verify no Managed Artifact writes happen.

### Blocked by

- Slice 1.

## 2.5. Project Root Discovery And Init [done]

**Type:** AFK

### What to build

Replace the unsafe cwd-as-Project-Root fallback with anchored Project Root discovery, explicit `--project-root`, mutation guards, and an `agentcfg init` workflow for non-git Projects.

### Acceptance criteria

- [ ] Project Root discovery walks ancestors from cwd and resolves git repository root before project marker root.
- [ ] Project markers include Shared Project Config, User Project Config, and the project-local configuration directory under the Project Root.
- [ ] Project Level workflows accept `--project-root` to override automatic discovery.
- [ ] Project Level mutation workflows block before writing when no Project Anchor exists unless `--project-root` is supplied.
- [ ] Unanchored mutation blockers include structured Diagnostics and Suggested Actions for `agentcfg init` or `--project-root`.
- [ ] `agentcfg init` establishes project markers at the chosen Project Root without writing lockfile pins or Managed Artifacts.
- [ ] `init` is idempotent when project markers already exist at the target Project Root.
- [ ] Project Level read-only workflows do not create project markers or Agent Configuration Files when unanchored.
- [ ] Read-only workflows may report an unanchored empty state with a Diagnostic when no anchor is found and `--project-root` is omitted.

### Testing scope

- Table-driven core tests for git root discovery, marker root discovery, explicit override, and unanchored directories.
- Core tests that mutation workflows block before write when unanchored.
- Core and CLI tests for `init` creating markers and enabling subsequent Project Level mutations in non-git fixtures.
- CLI tests for `--project-root`, blocker exit codes, and text/JSON Diagnostics.
- Do not shell out to the git binary in unit tests unless using isolated fixture repositories.

### Blocked by

- Slice 2.

## 3. Select And Deselect Explicit Skills

**Type:** AFK

### What to build

Implement explicit Skill Configuration mutation for `agentcfg skills select` and `agentcfg skills deselect`, limited to explicit Included Skills and existing Default Client Selection inheritance.

### Acceptance criteria

- [ ] `skills select` writes only the selected Config Layer.
- [ ] `skills select` does not accept client selection.
- [ ] New Skill Configuration Entries inherit the Config Layer Default Client Selection.
- [ ] Selecting a Skill fails before writing when the entry would have no final client selection.
- [ ] Selecting an explicit Included Skill appends to a compatible existing Skill Configuration Entry.
- [ ] Incompatible selections create distinct entries.
- [ ] Optional Skill Configuration Entry Ids persist on `[[skills]]` rows and are unique within a Config Layer when declared.
- [ ] `skills select` and `skills deselect` accept `--id` to target an entry by Skill Configuration Entry Id; locator-based selection remains available when no Entry Id is declared.
- [ ] `skills deselect` removes explicit Included Skills from the selected Config Layer only.
- [ ] Deselecting from User Project Config does not hide Skill Configuration from Shared Project Config.
- [ ] Mutation output tells the User to run Install, and Deselect Skill output also mentions Prune for stale Managed State.

### Testing scope

- Core tests for Select Skill and Deselect Skill mutation behavior.
- CLI tests for `skills select`, `skills deselect`, follow-up Suggested Actions, text/JSON output, and exit codes.
- Do not perform Source Enumeration, Source Resolution, lockfile writes, or Managed Artifact writes.

### Blocked by

- Slice 2.
- Slice 2.5.

## 4. Full Config Show For Skill Configuration

**Type:** AFK

### What to build

Complete authored-configuration inspection for V1 Skill Configuration, including Default Client Selection, entry-level clients, Skill Sources, explicit and all-skill Skill Selection syntax, Excluded Skills, and Skill Aliases.

### Acceptance criteria

- [ ] `agentcfg config show` defaults to Project Level and reports all active Project Config Layers partitioned by Config Layer.
- [ ] `--config-layer` narrows inspection to one Agent Configuration File.
- [ ] `--level user` inspects User Config.
- [ ] Config Show validates TOML, schema, local shape, client names, duplicate fields, and invalid or mixed selection syntax.
- [ ] Config Show renders `include = ["source-skill-name"]` and `include = "all"` without expanding source contents.
- [ ] Config Show rejects `include = "*"` and mixed selection modes.
- [ ] Config Show does not read Agent Configuration Lockfiles.
- [ ] Config Show does not perform Source Enumeration, Source Resolution, git access, or network access.

### Testing scope

- ConfigDoc parsing and writing tests for V1 Skill Configuration fields.
- Core Config Show tests for partitioned layers, narrowing, User Level, and validation.
- CLI output and exit code tests for text and JSON.
- Tests should assert read-only behavior by verifying no lockfile, Managed State, or Client Search Location mutation.

### Blocked by

- Slice 3.

## 5. Common Agent Skill Client Adapter Tracer

**Type:** HITL - needs exact first Client Search Location and Client Binding Artifact shape confirmation.

### What to build

Create the shared Client Adapter implementation for Clients that consume the Agent Skill Format in the same way, plus the minimal first Client descriptor needed by Preview and Install. Start with one concrete Client so later materialization has a real target, while keeping common Agent Skill Format behavior shared.

### Acceptance criteria

- [ ] The Client Adapter Catalog has a descriptor model that separates shared Agent Skill Format behavior from per-Client search locations and artifact shapes.
- [ ] One first Client descriptor is implemented after its Project Level and User Level Client Search Locations are confirmed.
- [ ] The common adapter can derive one Client Binding and plan one Client Binding Artifact for one Managed Skill Tree.
- [ ] Client Adaptation remains separate from Skill Alias behavior.
- [ ] Unsupported configured item and Client combinations fail validation before PinnedConfig creation.

### Testing scope

- Unit tests for common Agent Skill Format adapter behavior.
- Contract tests for the first Client descriptor.
- ConfigRequest normalization tests for unsupported configured item and Client combinations.
- Do not add the full V1 Client list yet.

### Blocked by

- Slice 4.
- Human confirmation of the first Client descriptor's Client Search Locations and Client Binding Artifact shape.

## 6. Local Source Preview Tracer

**Type:** AFK

### What to build

Implement the first source-backed Preview path for non-git local filesystem Skill Sources with explicit Included Skills, Source Enumeration, Source Resolution, deterministic Content Hashing, in-memory missing-pin planning, and Client Binding derivation through the first Client Adapter.

### Acceptance criteria

- [ ] Source Enumeration accepts a single root `SKILL.md`.
- [ ] Source Enumeration accepts child directories containing `SKILL.md`.
- [ ] A Skill Source containing both root and child-directory shapes fails validation as ambiguous.
- [ ] Source Skill Names come from valid `SKILL.md` frontmatter names.
- [ ] Missing, invalid, or duplicate frontmatter names fail validation.
- [ ] Non-git local filesystem Source Resolution records normalized source path and algorithm-qualified Content Hash.
- [ ] Content Hash excludes mutable filesystem metadata.
- [ ] Preview defaults to Project Level and supports explicit `--level user`.
- [ ] ConfigRequest normalization produces a final client selection from entry-level clients or Default Client Selection before Source Resolution.
- [ ] Final client selections whose Clients are not yet present in the Client Adapter Catalog fail validation before PlannedPinnedConfig creation.
- [ ] Preview resolves missing Agent Configuration Lockfile entries in memory, grouped by owning Config Layer.
- [ ] Preview produces a PlannedPinnedConfig with Configured Item Names and Client Bindings for explicit Included Skills.
- [ ] Duplicate Configured Item Names for explicit Included Skills across Active Config Layers fail validation before PlannedPinnedConfig creation.
- [ ] Preview reports blockers when sources needed for missing-pin resolution are unavailable.
- [ ] Preview writes nothing.

### Testing scope

- Filesystem fixture tests for root-only and child-directory-only Skill Sources.
- Deterministic Content Hash tests.
- Core Preview tests for final client selection normalization, unsupported Clients, missing pins, planned lockfile changes, Client Binding derivation, duplicate Configured Item Names, blockers, and no writes.
- CLI tests for Preview text/JSON output and exit behavior.

### Blocked by

- Slice 5.

## 7. Install One Local Skill To One Common-Adapter Client

**Type:** AFK

### What to build

Implement the first Install path: one explicit local filesystem Skill selected for one Client using the common Agent Skill adapter, with lockfile persistence, Managed Skill Tree preparation, Client Binding Artifact creation, and Preview parity for the same local Skill path.

### Acceptance criteria

- [ ] Install defaults to Project Level and supports explicit `--level user`.
- [ ] Install persists missing PinnedConfig entries to Agent Configuration Lockfiles before artifact writes.
- [ ] Lockfile writes are grouped by owning Config Layer.
- [ ] A Managed Skill Tree is prepared under the active Install Level's Managed State.
- [ ] The Managed Skill Tree preserves supporting files.
- [ ] One Client Binding Artifact is created or repaired for the selected Client only when the path is missing or Derived Ownership proves the existing artifact is managed.
- [ ] An Unmanaged Conflict at the required Client Binding Artifact path blocks mutation instead of being overwritten.
- [ ] Preview for the local Skill path reports the same Agent Configuration Lockfile, Managed Skill Tree, Client Binding Artifact, and Unmanaged Conflict changes or blockers that Install would act on, while writing nothing.
- [ ] Progress Events are written to stderr in text mode.
- [ ] JSON output writes no Progress Events and keeps stdout machine-readable.
- [ ] Install never writes changes back to Skill Sources.

### Testing scope

- Core Install tests for lockfile writes, Managed Skill Tree preparation, binding creation, repair of proven managed bindings, Unmanaged Conflict blocking, Preview/Install parity, and no source mutation.
- CLI tests for text/JSON output, stderr Progress Events, and exit codes.
- Fixture tests for Project Level and User Level isolation.

### Blocked by

- Slice 5.
- Slice 6.

## 8. Status For The Installed Local Skill Tracer

**Type:** AFK

### What to build

Implement Status for the installed local Skill path by comparing LockfilePinnedConfig with ObservedInstallation without source or git access.

### Acceptance criteria

- [ ] `agentcfg status` defaults to all relevant Install Levels.
- [ ] `--level project` and `--level user` narrow the report.
- [ ] Results are partitioned by Install Level.
- [ ] Missing Agent Configuration is reported as a normal empty state.
- [ ] Missing Agent Configuration Lockfile reports that expected state cannot be computed and suggests Install.
- [ ] Missing lockfiles are not reported as installed or not installed.
- [ ] Existing lockfiles with missing pins report Install required.
- [ ] Pinned-but-absent Managed Skill Trees or Client Binding Artifacts report installation drift.
- [ ] Status performs no Source Enumeration, Source Resolution, git access, network access, or source reachability checks.

### Testing scope

- Core Status tests for consistent installation, missing lockfile, missing pin, missing Managed Skill Tree, missing Client Binding Artifact, and broken binding artifact.
- CLI tests for text/JSON output and action-required nonzero exits.

### Blocked by

- Slice 7.

## 9. Basic Git Source Tracer

**Type:** AFK

### What to build

Add the earliest git-backed source path for manual testing: install one explicit Skill from a local git repository Skill Source without implementing GitHub shorthand, full git URLs, or the full Git Source Ref matrix yet.

### Acceptance criteria

- [ ] Local git repository Skill Sources are supported end-to-end through Preview and Install.
- [ ] Git-backed Source Resolution records repeatable source identity in PinnedConfig.
- [ ] The git-backed tracer reuses existing Source Enumeration, selection, Install, and Status behavior.
- [ ] Git failures surface structured Diagnostics and blockers.
- [ ] GitHub shorthand, full git URLs, remote fetching, and the full Git Source Ref matrix remain deferred to Slice 13.

### Testing scope

- Core tests with local git fixtures.
- Boundary tests should mock or isolate git command execution where remote access is not required.
- Do not cover GitHub shorthand, full URL, branch, tag, commit, remote fetching, or remote failure cases yet.

### Blocked by

- Slice 8.

## 10. All-Skill Selection, Exclusions, Aliases

**Type:** AFK

### What to build

Complete Skill Selection breadth for all enumerated Skills, Excluded Skills, Skill Aliases, duplicate Configured Item Name validation beyond the explicit-skill tracer, and alias-aware Managed Skill Tree preparation.

### Acceptance criteria

- [ ] `include = "all"` selects all enumerated Source Skill Names.
- [ ] Excluded Skills are valid only with all-skill selection.
- [ ] Excluded Skills must match enumerated Source Skill Names.
- [ ] Exclusions are entry-local and apply before Skill Aliases produce Configured Item Names.
- [ ] Skill Alias keys must match Source Skill Names that remain selected.
- [ ] Aliases for unselected or excluded Source Skill Names fail validation.
- [ ] Pattern, prefix, or default alias rules are rejected.
- [ ] Duplicate Configured Item Name validation covers all-skill selection and aliases across Active Config Layers before PinnedConfig or InstallPlan creation.
- [ ] Alias preparation makes the Managed Skill Tree directory and `SKILL.md` frontmatter `name` match the Configured Item Name.

### Testing scope

- Source Enumeration and selection validation tests for all-skill selection, exclusions, aliases, and duplicate names.
- Install tests for alias-aware Managed Skill Tree preparation.
- Tests must verify source files are not modified.

### Blocked by

- Slice 7.

## 11. Expand Common Adapter Catalog

**Type:** AFK/HITL - AFK after Client Search Locations and artifact shapes are known.

### What to build

Expand the Client Adapter Catalog for Codex, Pi, OpenCode, Claude Code, Cline, and Cursor, reusing the common Agent Skill Format implementation wherever Clients process Skills the same way.

### Acceptance criteria

- [ ] Each V1 Client has a descriptor for supported Install Levels, Client Search Locations, and Client Binding Artifact shape.
- [ ] Clients with the same Agent Skill Format behavior share the common adapter implementation.
- [ ] Per-Client code is limited to differences in search locations, artifact shape, and support rules.
- [ ] Project Level and User Level Client Search Locations are isolated.
- [ ] Unsupported Client or Install Level combinations return structured Diagnostics.
- [ ] One selected Skill with multiple supported Clients derives multiple Client Bindings.

### Testing scope

- Adapter contract tests for Codex, Pi, OpenCode, Claude Code, Cline, and Cursor.
- Install tests for multiple Client Bindings across supported Clients.
- Do not duplicate common adapter behavior in every Client test.

### Blocked by

- Slice 10.
- Human confirmation for any Client descriptor whose search locations or artifact shape are not already decided.

## 12. Entry-Level Client Workflows

**Type:** AFK

### What to build

Implement Skill Configuration Entry client workflows and derive updated Client Bindings from one selected Skill when an entry's client selection changes.

### Acceptance criteria

- [ ] `agentcfg skills clients show` reports entry-level and inherited client selections partitioned by Config Layer.
- [ ] `skills clients set` replaces entry-level clients.
- [ ] `skills clients add` and `remove` create explicit entry-level clients when the entry inherited the Default Client Selection.
- [ ] `skills clients inherit` removes entry-level clients so the entry inherits Default Client Selection again.
- [ ] Entry selection uses `--id` when declared, or explicit selectors such as `--source`, `--skill`, and `--ref` when needed.
- [ ] `--skill` matches Source Skill Name, not Skill Alias.
- [ ] Zero-match and multiple-match selectors fail with enough entry detail for disambiguation.
- [ ] Changing one Skill Configuration Entry's client selection updates the Client Bindings derived from that entry without changing other entries.
- [ ] Mutations do not change Managed Artifacts and tell the User to run Install.

### Testing scope

- Core tests for entry selection, inheritance transitions, mutation semantics, and updated Client Binding derivation.
- CLI tests for show/set/add/remove/inherit, text/JSON output, diagnostics, and exit codes.

### Blocked by

- Slice 11.

## 13. Full Git Source Coverage

**Type:** AFK

### What to build

Complete git-backed Skill Source support across local git repositories, GitHub shorthand, full git URLs, and Git Source Refs.

### Acceptance criteria

- [ ] Local git repositories resolve like other git-backed Skill Sources after path normalization.
- [ ] GitHub shorthand Skill Sources are accepted.
- [ ] Full git URL Skill Sources are accepted.
- [ ] Branches, tags, commits, labels that resolve to git refs, and other git-accepted refs are supported as Git Source Refs.
- [ ] Source Resolution records repeatable resolved refs and content identities.
- [ ] Source Enumeration and selection validation work consistently across git-backed source forms.
- [ ] Remote source access failures surface structured Diagnostics and blockers.
- [ ] Authentication UX remains out of scope beyond reporting source access failures and using the User's existing git environment.

### Testing scope

- Local git repository fixture tests.
- Mocked git and network boundary tests for GitHub shorthand and full git URLs.
- CLI tests for diagnostics and exit codes.

### Blocked by

- Slice 9.

## 14. Source Refresh And Changed Local Sources

**Type:** AFK

### What to build

Implement explicit Source Refresh for Preview and Install, plus changed-source blocking for locked non-git local filesystem sources when Source Refresh is not requested.

### Acceptance criteria

- [ ] Source Refresh is accepted on Preview and Install.
- [ ] Missing pins are resolved even without Source Refresh.
- [ ] Preview keeps refreshed PlannedPinnedConfig in memory and writes nothing.
- [ ] Install persists refreshed Agent Configuration Lockfile changes before artifact writes.
- [ ] Install without Source Refresh installs locked content when required Managed Skill Trees exist.
- [ ] Install without Source Refresh may rebuild missing Managed Skill Trees from local filesystem sources only when the current Content Hash matches the LockfilePinnedConfig.
- [ ] Changed local filesystem sources without Source Refresh block with Diagnostics containing Source Kind, normalized path, expected hash, and observed hash.
- [ ] Missing or inaccessible sources required for missing-pin resolution or Source Refresh block without writing to item sources.

### Testing scope

- Core Preview and Install tests for missing pins, refreshed pins, changed local sources, and inaccessible sources.
- Tests should verify write ordering: lockfile changes before artifact writes.
- CLI tests for Source Refresh option, diagnostics, and exit codes.

### Blocked by

- Slice 13.

## 15. Install Reconciliation And Conflict Safety

**Type:** AFK

### What to build

Complete InstallPlan construction and execution for repaired bindings, stale active-level Client Binding Artifact cleanup, stale Managed State reporting, inactive-level isolation, and Unmanaged Conflict blocking.

### Acceptance criteria

- [ ] Installation planning compares PlannedPinnedConfig with ObservedInstallation.
- [ ] Required Client Binding Artifacts are created or repaired.
- [ ] Stale Client Binding Artifacts for the active Install Level are removed only when ownership can be derived.
- [ ] Inactive Install Level Managed Artifacts are not mutated.
- [ ] Unmanaged Conflicts block mutation instead of being overwritten.
- [ ] Install reports lockfile updates grouped by owning Config Layer.
- [ ] Install reports Managed Artifact changes grouped by Install Level and Client.
- [ ] Install does not run Prune.
- [ ] Stale Managed Skill Trees are left for Prune.

### Testing scope

- Core planning tests for missing artifacts, stale Client Binding Artifacts, stale Managed State artifacts, broken symlinks, unexpected binding targets, and Unmanaged Conflicts.
- Install execution tests for active-level mutation and inactive-level isolation.
- CLI output tests for grouped changes, skipped artifacts, and blockers.

### Blocked by

- Slice 12.
- Slice 14.

## 16. Conservative Prune

**Type:** AFK

### What to build

Implement Prune using Derived Ownership and Conservative Prune for stale Managed State and stale Client Binding Artifacts at one active Install Level.

### Acceptance criteria

- [ ] Prune defaults to Project Level and supports explicit `--level user`.
- [ ] Prune removes Stale Managed Artifacts only when ownership can be derived.
- [ ] Prune can remove stale Managed State with missing lockfiles only when ownership is derived from non-lockfile evidence.
- [ ] Missing lockfiles reduce ownership evidence but do not by themselves permit or forbid cleanup.
- [ ] Prune reports removed stale Managed State artifacts by Install Level.
- [ ] Prune reports removed stale Client Binding Artifacts by Client.
- [ ] Prune reports skipped artifacts with safety reasons such as ownership not derived, Unmanaged Conflict, or inactive Install Level.
- [ ] Prune never removes Unmanaged Conflicts.
- [ ] Prune never mutates inactive Install Level artifacts.

### Testing scope

- Safety-critical, table-driven Core tests for Derived Ownership and Conservative Prune.
- CLI tests for grouped removals, skipped safety reasons, text/JSON output, and exit codes.

### Blocked by

- Slice 15.

## 17. Doctor

**Type:** AFK

### What to build

Implement Doctor as a local capability and configuration assessment workflow, partitioned by Install Level and optionally extended with source reachability checks.

### Acceptance criteria

- [ ] `agentcfg doctor` defaults to all relevant Install Levels.
- [ ] `--level project` and `--level user` narrow the report.
- [ ] Results stay partitioned by Install Level.
- [ ] Empty Install Levels are reported as normal empty states.
- [ ] Doctor validates environment capabilities, supported Clients, Config Layer readability, lockfile readability, Managed State accessibility, Client Search Location accessibility, and local source path availability.
- [ ] Remote git or network source reachability checks run only with `--check-sources`.
- [ ] Managed State without readable corresponding Agent Configuration or lockfile context is reported as a diagnostic.
- [ ] Orphaned Managed State is not classified as drift and is never removed by Doctor.
- [ ] Doctor exits zero only when all enabled checks pass or reported levels are empty.

### Testing scope

- Core Doctor tests for local checks, empty levels, unreadable paths, unsupported Clients, and orphaned Managed State diagnostics.
- Tests for `--check-sources` gating remote source checks.
- CLI tests for text/JSON output and exit codes.

### Blocked by

- Slice 16.
