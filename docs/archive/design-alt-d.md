# Alt-D: Immutable Workflow Pipeline

Input boundary: this alternative was derived from `docs/prd.md` and the requested architecture/maintainability/design-review principles only. It intentionally does not use existing design files, alternate designs, or repository code.

## Thesis

V1 is implemented as command-specific pipelines of immutable planning stages. Each stage consumes a snapshot and returns a richer snapshot, plan, or diagnostic set. Persistent mutation is concentrated in one executor module, and `preview` reaches the same planned outcome as `apply` without invoking that executor.

This design optimizes for locality around command flow: if a user-facing command changes, the command recipe is the first place to inspect. It avoids a generic controller loop and avoids making every PRD noun a top-level domain module unless that noun protects an invariant or reduces caller burden.

## Module Map

- `cli`: parses arguments into command requests and maps results to terminal output and exit codes.
- `workflow`: owns the command recipes for `init`, `preview`, `apply`, `prune`, `status`, and `doctor`.
- `pipeline`: provides a small `Input -> Output + Diagnostics` stage convention; it has no product policy.
- `layer_input`: locates and reads the active Config Layers for Project Level or User Level.
- `source_snapshot`: resolves path and git Skill Sources into immutable source snapshots.
- `skill_selection`: discovers Agent Skill Format directories and applies `include`, `groups`, and aliases.
- `lock_planning`: computes missing lockfiles, lockfile deltas, and Locked Desired State without writing.
- `content_planning`: plans Managed Skill Content and Discovery Name preparation.
- `client_projection`: expands locked skills into Client Discovery Location requirements.
- `state_snapshot`: reads the Manifest, Managed State, Managed Skill Content, and Client Discovery Locations.
- `diff_planning`: produces apply, prune, and status plans from desired and current snapshots.
- `safety_gate`: validates write and delete actions before execution.
- `executor`: the only module that writes config, lockfiles, manifest records, managed content, discovery artifacts, or removals.
- `reporting`: renders user-facing summaries and diagnostics.

The deep modules are `workflow`, `lock_planning`, `diff_planning`, and `safety_gate`. Deleting any of these would spread ordering rules, lifecycle distinctions, and cleanup safety across command handlers.

## Command Flow

`init`

1. Resolve the target Config Layer.
2. Build a config-template write plan.
3. Pass the plan through `safety_gate`.
4. Execute only the config creation.

`preview`

1. Read active Config Layers.
2. Snapshot Skill Sources.
3. Apply skill selection and aliases.
4. Compute lockfile deltas in memory.
5. Plan Managed Skill Content and Client Discovery Requirements.
6. Snapshot current managed install state.
7. Render lockfile changes, source resolutions, artifact creates/updates, requirement changes, stale state, alias preparation, and warnings.

`preview --refresh-sources` follows the same recipe, but `source_snapshot` refreshes moving refs or changed path content in memory.

`apply`

`apply` follows the same planning path as `preview`, then `executor` writes only:

- missing or refreshed lockfiles
- Managed Skill Content
- Installed Artifact creates/updates
- Discovery Requirement additions

It does not remove stale state. Stale Installed Artifacts or Stale Discovery Requirements are reported with the PRD warning to run `agentcfg prune`.

`prune`

1. Read active layer scope and client filters.
2. Snapshot Manifest and Client Discovery Locations.
3. Plan stale Discovery Requirement removals and stale Installed Artifact removals.
4. Refuse unsafe deletes in `safety_gate`.
5. Execute only manifest-owned cleanup.

`status`

`status` snapshots config, lockfiles, manifest, managed content, and discovery locations, then reports managed install-state consistency: broken symlinks, unexpected symlink targets, missing Managed Skill Content, stale artifacts, unsatisfied requirements, config/lock mismatch, manifest readability, and unmanaged artifacts.

`doctor`

`doctor` runs environment and configuration readiness checks: git availability, Project Root detection, supported clients, path writability, config schema validity, optional Skill Source/network checks, Client Discovery Location confidence, and unmanaged artifacts only when they block previewed paths. It does not reuse `status` as a general install-state check.

## Persistence Model

- Config files and lockfiles live beside their Config Layer as specified by the PRD.
- Lockfiles store Locked Desired State for repeatable Skill Source resolution.
- Managed State stores the Manifest and Managed Skill Content.
- The Manifest is the ownership ledger for Installed Artifacts and Discovery Requirements.
- Discovery Requirements are keyed by Config Layer, Client, Install Level, and artifact identity.
- Client Discovery Locations contain manifest-owned symlinks or copies to Managed Skill Content.
- Writes use temporary files plus rename where practical; Manifest writes happen after artifact writes succeed.

## Safety Invariants

- Immutable planning stages never mutate persistent state.
- `preview`, `status`, and `doctor` have no executor path.
- `apply` creates and updates only; stale cleanup belongs to `prune`.
- `prune` removes only manifest-owned artifacts.
- Unexpected symlink targets are refusal conditions.
- Unmanaged real files are never deleted.
- Directories are deleted only when empty and manifest-owned.
- Discovery Name Collisions fail before any write.
- `--client` only narrows configured clients unless config uses `clients = "all"`.
- User Config is never merged into Project Level apply.

## Testing Strategy

- Pure stage tests with fixture snapshots for config, sources, locks, manifest, and filesystem views.
- Command recipe tests proving read-only commands do not include the executor.
- Source adapter tests for path and git resolution, including refresh behavior.
- Golden preview tests for lock changes, artifact changes, stale state, alias preparation, and collisions.
- Filesystem integration tests for symlink materialization, external symlink rejection, broken links, unexpected targets, and shared `.agents/skills` requirements.
- Status/doctor separation tests so install consistency does not drift into doctor.

## Design Risks

- Too many intermediate plan types could become ceremony. Keep stages coarse and delete types that only forward fields.
- A generic pipeline runner could become a framework. Keep it small enough that command recipes remain readable.
- Git refresh can leak into callers. Hide it behind `source_snapshot`.
- Client support can invite premature plugin architecture. V1 should use a simple built-in registry until client behavior genuinely diverges.
- Lockfile and Manifest schemas are high-leverage persistence choices. Keep versioning local to persistence modules.

## What Not To Abstract

- Do not create a generic Configured Item platform in V1.
- Do not add a trait per client unless two clients require different behavior, not just different paths.
- Do not introduce a reconciler loop, background controller, event bus, or global plan language.
- Do not split every PRD noun into its own module.
- Do not abstract source catalogs, org layers, MCP, commands, rules, or UI.
- Do not add flags or modes beyond the PRD command surface.
