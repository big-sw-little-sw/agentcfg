# Agent Configuration Context

The Agent Configuration context owns repeatable, pinned configuration for agent-facing Configured Items.

## Responsibility

This context turns user-authored Agent Configuration into safe, repeatable managed installation for supported Clients.

## V1 Boundary

V1 implements Skill Configuration only. Future configured item kinds, such as subagent configuration and MCP configuration, must add item-kind-specific vocabulary and implementation behind the shared configuration, resolution, lockfile, planning, installation, diagnostics, and ownership language.

V1 Skill Configuration schema, command mechanics, source enumeration, selection, exclusion, aliasing, and client-selection details live in the V1 spec. This context keeps the durable Agent Configuration constraints.

## Invariants

### Workflow And Presentation Boundaries

- The Core Crate owns domain behavior and exposes workflows through the Workflow API.
- Presentations render inputs, Progress Events, Diagnostics, Suggested Actions, workflow results, and exit behavior from structured workflow outputs; they do not duplicate domain behavior.
- Human and machine-readable output are renderings of the same workflow result model, not separate reporting models.
- Diagnostics and Suggested Actions are structured workflow outputs. Suggested Actions may name mutating commands such as Install or Prune, but they are follow-up guidance only and are never executed automatically.

### Layers, Levels, And Ownership

- Project Root resolves from git repository root discovery, existing Project Markers, explicit `--project-root`, or `init`. Project Level mutation workflows require a Project Anchor and must not treat an unmarked current working directory as a writable Project.
- Config Layers own their Agent Configuration Files, Agent Configuration Lockfiles, and declarations. Lockfile writes and planned lockfile changes are attributed to the Config Layer that owns the declaration.
- Active Project Config Layers are additive. User Project Config may add declarations alongside Shared Project Config, but it may not override, subtract from, or mutate Shared Project Config.
- Project Managed State is shared by all active Project Config Layers. Shared Project Config and User Project Config keep separate files and lockfiles, but Project Level materialization prepares Managed Artifacts under the same Project Managed State root.
- Project Level and User Level are isolated for mutation. A workflow mutating one Install Level must not clean up or rewrite Managed Artifacts for another Install Level.
- Mutation workflows may change only Managed Artifacts whose ownership and Install Level can be derived. Missing lockfiles reduce ownership evidence, but do not by themselves authorize or forbid cleanup.

### Configuration And Materialization

- Config mutation workflows update Agent Configuration Files only. Install and Prune materialize configuration changes into Managed Artifacts.
- Preview is read-only, scoped to one Install Level, and shows what Install would change for that level without writing Agent Configuration Files, Agent Configuration Lockfiles, Managed State, item sources, or Client Search Locations.
- Install reconciles exposure artifacts for the active Install Level and installs required Managed Artifacts. Install does not run Prune; stale Managed State cleanup remains explicit Prune behavior.
- Prune applies Conservative Prune to stale Managed Artifacts only when ownership can be derived. Prune never removes Unmanaged Conflicts.
- Config Show is read-only authored-configuration inspection. It does not read lockfiles, perform Source Resolution, or validate source-dependent facts.

### Assessment

- Status and Doctor are read-only assessment workflows. Results must stay partitioned by Install Level and must not merge Project Level and User Level state.
- An Install Level with no Agent Configuration is a normal empty state for read-only assessment workflows, not a warning or blocker.
- Status compares lockfile-backed expected state with ObservedInstallation. Source availability is not installation drift, so Status does not perform Source Resolution or source reachability checks.
- Status distinguishes missing lockfiles, missing pins, and missing artifacts. A missing lockfile blocks expected-state comparison, a missing pin means Install is required, and only a pinned-but-absent artifact is reported as installation drift.
- Doctor reports local capability and configuration problems. It does not classify unmanaged or orphaned Managed State as drift and never removes it.

### Configured Items

- Public workflow, planning, lockfile, diagnostic, and ownership concepts are item-kind neutral. Skill-specific behavior must not leak into shared public workflow concepts.
- Configured item kinds define their own source enumeration, naming, selection, aliasing, content preparation, validation, and client adaptation behind the shared workflow model.
- A Configured Item Name must be unique across Active Config Layers after item-kind-specific naming or alias resolution, even when duplicate entries target different Clients.
- A Config Layer may provide Default Client Selection for configured item entries. Entry-level clients replace that default, and every entry must have a final client selection after ConfigRequest normalization.
- When declared, Skill Configuration Entry Ids must be unique within a Config Layer. Duplicate Entry Ids fail validation before mutation or PinnedConfig creation.
- Unsupported configured item and Client combinations fail validation before PinnedConfig creation.
