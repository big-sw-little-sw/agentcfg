# Alt-E: Bounded Declarative Reconciler

Input boundary: this alternative was derived from `docs/prd.md` and the requested architecture/maintainability/design-review principles only. It intentionally does not use existing design files, alternate designs, or repository code.

## Thesis

V1 is implemented as a bounded declarative reconciler. Each command builds a `DesiredResources` snapshot, reads a `CurrentResources` snapshot, and asks a concrete `reconciler` to produce an allowed `Delta`.

The reconciler is the main deep module. It owns lifecycle policy, stale detection, Discovery Name Collision handling, and cleanup safety distinctions. Commands are thin projections over reconciliation, not pipelines of reusable steps and not command-specific transaction scripts.

## Module Map

- `cli`: parses flags into command requests and renders command results.
- `layer_store`: reads and writes selected Config Layer config and lockfile files.
- `source_resolver`: resolves path and git Skill Sources into locked skill content identities.
- `client_registry`: maps configured clients and Install Level to Client Discovery Locations and confidence warnings.
- `desired_builder`: builds `DesiredResources` from active configs, lockfiles, refresh mode, aliases, selections, groups, and client filters.
- `current_inventory`: reads Manifest, Managed State, Managed Skill Content, and Client Discovery Locations into `CurrentResources`.
- `reconciler`: computes command-specific deltas from desired/current state.
- `delta_applier`: applies approved deltas in the required order.
- `reporter`: renders preview, apply, prune, status, and doctor output from deltas and diagnostics.

The earned interface is:

```text
reconcile(mode, desired, current, environment) -> ReconcileReport
```

Deleting `reconciler` would scatter stale handling, manifest ownership, cleanup refusal, command mode policy, and collision handling across callers. That makes it a real module rather than a pass-through.

## Resource Model

`DesiredResources` contains:

- lockfile entries needed for active Config Layers
- Managed Skill Content entries needed for locked skills
- Discovery Requirements keyed by Config Layer, Client, Install Level, and discovery path
- desired Installed Artifacts for Client Discovery Locations
- source-resolution diagnostics and client-location warnings

`CurrentResources` contains:

- existing lockfile state
- existing Managed Skill Content
- Manifest records
- observed discovery artifacts
- unmanaged artifacts, broken symlinks, and unexpected symlink targets

`Delta` contains concrete actions:

- lockfile creates/updates
- Managed Skill Content creates/updates
- Installed Artifact creates/updates
- Discovery Requirement additions
- stale Discovery Requirement removals
- stale Installed Artifact removals
- refusal diagnostics
- status findings

Resource kinds remain concrete for V1 skills. This is not a generic resource graph engine.

## Command Flow

`init`

`init` is the exception: it uses `layer_store` to create the selected Config Layer skeleton and does not run reconciliation.

`preview`

1. Build desired resources from config and locks.
2. Read current resources.
3. Run `reconcile(Preview, desired, current, environment)`.
4. Render the delta.
5. Perform no writes.

With `--refresh-sources`, refreshed source resolutions are included only in desired resources and proposed lockfile deltas.

`apply`

1. Build desired resources, refreshing sources if requested.
2. Read current resources.
3. Run `reconcile(Apply, desired, current, environment)`.
4. Apply only lockfile creates/updates, Managed Skill Content materialization, Installed Artifact creates/updates, and Discovery Requirement additions.
5. Report stale resources as warnings.

`prune`

1. Build desired resources for the active scope and client filters.
2. Read current resources.
3. Run `reconcile(Prune, desired, current, environment)`.
4. Apply only stale Discovery Requirement removals and stale Installed Artifact removals that pass safety checks.

`status`

`status` runs inventory-heavy reconciliation in `Status` mode and reports managed install-state consistency: Installed Artifacts, broken symlinks, unexpected symlink targets, missing Managed Skill Content, stale artifacts, unsatisfied requirements, config/lock mismatch, manifest readability, and informational unmanaged artifacts.

`doctor`

`doctor` is environment-oriented. It checks git availability, Project Root detection, supported clients, path writability, config schema validity, optional source/network checks, Client Discovery Location confidence, and blocking unmanaged artifacts. It does not report overall managed install consistency.

## Persistence Model

- Config and lockfiles are owned by `layer_store`.
- Manifest and Managed Skill Content are owned by Managed State.
- Manifest records `DiscoveryRequirement` and `InstalledArtifact` resources.
- Managed Skill Content is addressed by locked source identity, not live mutable source paths.
- Client Discovery Locations contain symlinks or copies to Managed Skill Content.
- `reconciler` treats persistence as snapshots plus deltas; only `delta_applier` mutates disk.

## Safety Invariants

- `preview` produces zero persistent writes.
- `apply` never removes stale artifacts or stale requirements.
- `prune` removes only manifest-owned resources.
- Unexpected symlink targets are refusal conditions.
- Unmanaged real files are never deleted.
- Directories are deleted only when empty and manifest-owned.
- Discovery Name Collisions are detected before any write.
- `--client` narrows configured clients only; it does not add clients outside configured selection unless config uses `clients = "all"`.
- User-level and project-level desired resources are built separately.

## Testing Strategy

- Unit-test `reconciler` with in-memory desired/current resource sets.
- Golden-test preview deltas for create, update, stale, collision, alias, and refresh cases.
- Integration-test `delta_applier` with temp directories for symlink creation, copy fallback if needed, broken symlink, unexpected target, external symlink rejection, and empty-directory pruning.
- Test refresh as a desired-resource change instead of a separate pipeline.
- Test shared `.agents/skills` across Codex, Pi, OpenCode, and Cursor as multiple requirements on one Installed Artifact.
- Test `status` and `doctor` separately so their questions remain distinct.

## Design Risks

- A generic reconciler framework would overfit V1. Keep resource kinds concrete and skill-oriented.
- Delta ordering can become hidden complexity. Put ordering in `delta_applier`, not in command handlers.
- `status` and `preview` need similar facts but different meanings. Keep command mode explicit.
- Source refresh can leak into many modules. Keep moving-ref and content-change logic inside `source_resolver`.
- Resource names can become pseudo-public contracts. Keep them internal until a future configured item proves the abstraction.

## What Not To Abstract

- Do not build a generic resource graph engine.
- Do not create a plugin system for future Configured Item kinds.
- Do not abstract client discovery beyond the V1 built-in registry.
- Do not expose internal resource IDs as user-visible contracts.
- Do not add reusable workflow steps; commands should stay thin projections over reconciliation.
- Do not add policy flags for cleanup beyond the PRD's `apply` versus `prune` split.

