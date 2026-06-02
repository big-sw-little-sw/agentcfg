# Recommended V1 Architecture

This is the recommended concrete V1 design.

It chooses [Alt-1](architecture-alt-1-skill-first-shared-operations.md) as the
primary shape and borrows two narrow ideas:

- From [Alt-2](architecture-alt-2-layer-contract-compiler.md): preserve Config
  Layer provenance in every record that crosses a Seam.
- From [Alt-3](architecture-alt-3-workflow-kernel.md): make mutation rules
  structural, so Preview cannot write and Apply/Prune recheck preconditions
  immediately before writes.

It does not adopt Alt-2's central Layer Contract compiler or Alt-3's central
Workflow Kernel. Both introduce too much Interface before V1 has more than one
Configured Item kind.

## Decision

Adopt a skill-first resolver with shared Installed Artifact operation planning.

```text
Skill Configuration + Skill Sources
  -> prepared skill facts
  -> Desired State
  -> OperationPlan
  -> Preview rendering or Apply/Prune execution
```

The external Seam is `workflow`: CLI handlers call core workflow functions and
render structured results.

The important internal Seam is between skill-specific resolution and shared
operation planning. V1 has one Adapter at that Seam, Skill, so the Seam stays
concrete and narrow.

## Recommended Module Layout

```text
crates/agentcfg-cli/src/
  main.rs
  args.rs
  commands.rs
  render.rs

crates/agentcfg-core/src/
  workflow/
    mod.rs
    context.rs
    install_context.rs
    init.rs
    preview.rs
    apply.rs
    prune.rs
    status.rs
    doctor.rs
    types.rs

  config.rs
  config_paths.rs
  layer_level.rs

  skill_source/
    mod.rs
    path.rs
    git.rs
    groups.rs
    selection/
      mod.rs
      aliases.rs
      diagnostics.rs

  skill_content/
    mod.rs
    hashing.rs

  managed_skill_content/
    mod.rs

  lockfile.rs

  discovery_registry/
    mod.rs
    resolution.rs

  desired_state/
    mod.rs
    records.rs
    skill.rs

  manifest.rs

  install_health/
    mod.rs
    records.rs

  operations/
    mod.rs
    records.rs
    preview.rs
    apply.rs
    prune.rs

  execution/
    mod.rs
    apply.rs
    prune.rs
```

Names can change during implementation. Ownership should not.

## Module Ownership

| Module | Interface | Ownership |
| --- | --- | --- |
| `agentcfg-cli` | command handlers and renderers | Argument parsing, terminal output, exit codes. |
| `workflow` | public request/result functions | Thin orchestration only. |
| `workflow::install_context` | `build_install_context` | Install Level, Active Config Layers, paths, Project Root, User paths, Managed State roots. |
| `config` | load and validate Skill Configuration | TOML schema and Persisted Scope Value validation. |
| `skill_source` | resolve Discovery Name-bearing Skill Selection | Path/git acquisition, discovery, Skill Groups, Included Skills, Skill Alias output. |
| `skill_content` | prepare skill content facts | Safe tree walk, Discovery Name preparation, hashing, symlink and special-file policy. |
| `managed_skill_content` | read/write Managed Skill Content | Content-addressed prepared content under Managed State. |
| `lockfile` | read/write Locked Desired State | Deterministic per-Config Layer persistence. |
| `discovery_registry` | resolve Client Discovery Locations | Client catalog, `clients = "all"`, `--client` narrowing, confidence/provenance. |
| `desired_state` | assemble Desired State | Desired Installed Artifacts and Discovery Requirements. |
| `manifest` | read/write Manifest records | Installed Artifact ownership and Discovery Requirement persistence. |
| `install_health` | classify current state | Unmanaged Artifact, Broken Symlink, Unexpected Symlink Target, stale and unsatisfied facts. |
| `operations` | build OperationPlan | Discovery Name Collision, creates, updates, stale reports, prune candidates, warnings. |
| `execution` | execute Apply/Prune writes | Mutation and precondition rechecks. |

## Core Records

These records are the shared language between Modules.

```rust
struct InstallLevelContext {
    install_level: InstallLevel,
    active_layers: Vec<ConfigLayer>,
    config_paths_by_layer: BTreeMap<ConfigLayer, PathBuf>,
    lockfile_paths_by_layer: BTreeMap<ConfigLayer, PathBuf>,
    managed_state_paths: ManagedStatePaths,
    project_root: Option<PathBuf>,
    user_dirs: UserDirs,
}

struct DiscoveryNamedSkill {
    config_layer: ConfigLayer,
    skill_source_id: String,
    source_skill_name: String,
    discovery_name: String,
    skill_directory: PathBuf,
}

struct PreparedSkillContent {
    config_layer: ConfigLayer,
    skill_source_id: String,
    source_skill_name: String,
    discovery_name: String,
    source_hash: Hash,
    installed_hash: Hash,
    discovery_name_prepared: bool,
    materialization_version: u32,
}

struct LockedSkill {
    config_layer: ConfigLayer,
    skill_source_id: String,
    skill_source_type: SkillSourceType,
    requested_git_ref: Option<String>,
    resolved_git_commit: Option<String>,
    source_skill_name: String,
    discovery_name: String,
    source_hash: Hash,
    installed_hash: Hash,
    discovery_name_prepared: bool,
    materialization_version: u32,
}
```

```rust
struct ResolvedClientDiscoveryLocation {
    configured_item_kind: ConfiguredItemKind,
    client: ClientId,
    install_level: InstallLevel,
    path: PathBuf,
    confidence: DiscoveryLocationConfidence,
    provenance: DiscoveryLocationProvenance,
}

struct DiscoveryRequirement {
    config_layer: ConfigLayer,
    client: ClientId,
    install_level: InstallLevel,
}

struct DesiredInstalledArtifact {
    configured_item_kind: ConfiguredItemKind,
    client_discovery_location: PathBuf,
    discovery_name: String,
    managed_skill_content_path: PathBuf,
    installed_hash: Hash,
    skill_provenance: SkillProvenance,
    discovery_requirements: Vec<DiscoveryRequirement>,
}

struct ManifestInstalledArtifact {
    configured_item_kind: ConfiguredItemKind,
    client_discovery_location: PathBuf,
    discovery_name: String,
    installed_artifact_path: PathBuf,
    expected_symlink_destination: PathBuf,
    installed_hash: Hash,
    skill_provenance: SkillProvenance,
    discovery_requirements: Vec<DiscoveryRequirement>,
}
```

```rust
struct OperationPlan {
    lockfile_changes: Vec<LockfileChange>,
    managed_skill_content_changes: Vec<ManagedSkillContentChange>,
    installed_artifact_creates: Vec<DesiredInstalledArtifact>,
    installed_artifact_updates: Vec<DesiredInstalledArtifact>,
    discovery_requirement_additions: Vec<DiscoveryRequirement>,
    stale_reports: Vec<StaleReport>,
    prune_candidates: Vec<PruneCandidate>,
    warnings: Vec<OperationWarning>,
}
```

## End-To-End Flow

1. `workflow` receives a request.
2. `workflow::install_context` resolves Install Level, Active Config Layers,
   config paths, Lockfile paths, Managed State paths, Project Root, and User
   paths.
3. `config` loads and validates Active Config Layers.
4. `lockfile` reads adjacent Lockfiles.
5. `skill_source` resolves Skill Sources using `UseLocked` or
   `RefreshSources`, expands Skill Selection, applies Skill Aliases, and emits
   Discovery Name-bearing skill facts.
6. `skill_content` safely walks content, prepares Discovery Name metadata, and
   computes `source_hash` and `installed_hash`.
7. `managed_skill_content` computes intended Managed Skill Content paths. Only
   Apply writes them.
8. `discovery_registry` resolves configured Clients and CLI `--client`
   narrowing.
9. `desired_state` creates Desired State and Discovery Requirements.
10. `manifest` reads current ownership records.
11. `install_health` classifies current state.
12. `operations` detects Discovery Name Collision, stale state, creates,
    updates, and warnings.
13. Preview renders the OperationPlan and performs no writes.
14. Apply writes Lockfiles, Managed Skill Content, Installed Artifacts, and
    Manifest updates. It does not remove stale state.
15. Prune removes only stale manifest-owned state after precondition checks.
16. Status and Doctor reuse structured facts rather than duplicating scans.

## Workflow Semantics

### Preview

Preview builds the same OperationPlan that Apply uses, but receives no
write-capable Adapter. `preview --refresh-sources` refreshes Skill Source
resolution in memory only.

### Apply

Apply uses Locked Desired State unless Source Refresh is requested. It writes
missing or refreshed Lockfiles, materializes Managed Skill Content, creates or
updates manifest-owned Installed Artifacts, and records Discovery Requirements.

Apply warns about Stale Discovery Requirements and Stale Installed Artifacts. It
does not remove them.

### Prune

Prune recomputes Desired State from active configuration and Locked Desired
State, reads the Manifest, classifies install health, and removes only stale
manifest-owned state after rechecking preconditions.

### Status

Status answers whether the current managed install state is consistent for an
Install Level.

### Doctor

Doctor answers whether the local environment and configuration are capable of
working. It does not replace Status.

## Persistence

Lockfiles record Locked Desired State per Config Layer and do not store
machine-local Client Discovery Location paths.

The Manifest records ownership under Managed State:

- Configured Item kind
- Client Discovery Location
- Discovery Name
- Installed Artifact path
- expected symlink destination
- installed hash
- Skill Source provenance
- Discovery Requirements

Managed Skill Content should be content-addressed under Managed State. The exact
path formula can be implementation-specific if it remains stable for Manifest
records and diagnostics.

## Safety Rules

- Preview is read-only.
- Apply is one-way from Skill Source to Managed Skill Content to Client
  Discovery Location.
- Apply never writes Skill Sources.
- Apply never removes stale Managed State.
- Prune is the only workflow that removes Stale Discovery Requirements or Stale
  Installed Artifacts.
- Lockfile is the repeatability authority for Locked Desired State.
- Manifest is the ownership authority for Installed Artifacts and Discovery
  Requirements.
- Client Discovery Registry is the location authority for Client Discovery
  Locations.
- Discovery Name Collision is detected per Client Discovery Location before
  mutation.
- Unmanaged Artifacts are never overwritten or deleted.
- Unexpected Symlink Target blocks update and prune.
- Apply and Prune recheck filesystem preconditions immediately before writing.
- Config Layer, Install Level, and Persisted Scope Value remain distinct.

## Reserved For Post-V1

Reserve these now:

- `ConfiguredItemKind::Skill` in Desired State, Manifest, operation records, and
  Client Discovery Registry keys.
- Aggregate command meaning: `agentcfg apply` applies Configured Items for the
  selected Install Level; V1 only has Skills.
- Kind-aware operation records, without generic Configured Item manager traits.
- Discovery Requirements independent of Skill Selection reason.
- Client Discovery Registry keyed by Configured Item kind.

Defer these:

- generic Configured Item config schema
- generic resolver/applier traits, factories, or dynamic registration
- cross-kind transaction or rollback behavior
- MCP, hooks, rules, commands, workflows, and native client config edits
- direct Skill Source symlink mode
- user-configurable Installed Artifact mode
- Unmanaged Artifact adoption/import
- replacement or precedence semantics between Config Layers

## Extensibility Without Central Alt-2 Or Alt-3

The recommended design is intentionally not a full Layer Contract compiler and
not a full Workflow Kernel. It is still extendable for the next few Configured
Item kinds if new kinds enter through the same narrow pattern:

```text
kind-specific resolver
  -> Desired State records
  -> OperationPlan
  -> execution with precondition checks
```

The important rule is to add each new Configured Item kind as a concrete Module
first. Extract a broader Interface only after repeated behavior appears across
two or more real kinds.

Likely fit by future kind:

| Future kind | Fit with recommended design | Expected new ownership |
| --- | --- | --- |
| Rules | Good if rules are files or directories discovered from Client Discovery Locations. | A rules resolver that validates rule declarations and emits Desired State records. |
| Agent definitions | Good if agent definitions are managed files or directories, with possible cross-reference checks. | An agent-definition resolver that validates references to Skills, MCP servers, or rules before Desired State assembly. |
| MCP | Partial. MCP may need native client config edits, secrets posture, merge policy, or JSON/TOML mutation. | An MCP resolver and MCP-specific operation records if Installed Artifact records are not expressive enough. |

This means the next few kinds do not require a central Workflow Kernel. They do
require the records already reserved above: `ConfiguredItemKind`, kind-aware
Desired State, kind-aware Manifest records, kind-aware OperationPlan entries,
and Client Discovery Registry lookup by Configured Item kind.

### Why Not Central Workflow Kernel Yet

Alt-3's central Workflow Kernel becomes attractive when Apply needs coordinated
transitions across several kinds, such as:

```text
apply MCP server
  -> update agent definition
  -> update rules
  -> roll back or report partial progress if a native client config edit fails
```

V1 does not have that shape. V1 has one Configured Item kind and one dominant
mutation pattern: create or update manifest-owned Installed Artifacts from
Managed Skill Content. A central Workflow Kernel would create a broad Interface
before the Implementation has enough variation to justify it.

The V1 design should borrow only the Alt-3 safety idea:

- Preview receives no write-capable Adapter.
- Apply and Prune enter `execution`.
- `execution` rechecks filesystem preconditions immediately before mutation.
- Prune remains a distinct workflow, not Apply with deletions enabled.

Promotion rule: introduce a Workflow Kernel only when multiple Configured Item
kinds need coordinated transition ordering, partial-failure reporting, or
cross-kind rollback semantics that would otherwise be duplicated across
workflow Modules.

### Why Not Central Layer Contract Compiler Yet

Alt-2's central Layer Contract compiler is useful sooner than a Workflow Kernel,
but it should still remain a discipline rather than the main V1 Module.

Keep these Alt-2 invariants now:

- Config Layer provenance is present on records crossing Seams.
- Install Level is resolved once by `workflow::install_context`.
- Lockfile adjacency is handled in one place.
- Source Refresh policy is explicit.
- Desired State is built from Active Config Layers and Locked Desired State,
  not from the Manifest.

Do not centralize every kind through a `LayerContract` yet. With only Skills in
V1, that would risk turning Skill behavior into data shuffled through a large
intermediate record. Skill Source resolution, Skill Selection, Skill Alias
handling, and Managed Skill Content preparation would lose Locality.

Alt-2 becomes more attractive when Config Layer composition becomes the shared
complexity across several kinds:

- MCP declarations need layer-sensitive secrets or environment references.
- Rules need clear additive versus replacement behavior across Config Layers.
- Agent definitions reference Skills, MCP servers, or rules from different
  Config Layers.
- More than one kind needs the same Lockfile/Source Refresh/provenance logic.

Promotion rule: introduce a Layer Contract compiler only after a second or third
Configured Item kind proves that Config Layer composition and Lockfile
reconciliation are being duplicated. Until then, keep provenance explicit and
let each kind own its concrete resolver.

## Implementation Plan Alignment

The implementation plan should point at this recommendation near the top, while
remaining milestone-oriented.

Suggested alignment:

- M2.4 finishes Discovery Name-bearing Skill Selection.
- M3 owns `skill_content`: safe walk, symlink policy, hashing, and Discovery
  Name preparation.
- M4 owns `lockfile`, install-level context, and Managed Skill Content.
- M5 owns `discovery_registry`, `desired_state`, and `operations`.
- M6 owns `manifest`, `install_health`, and `execution`.
- M7 reuses `install_health` for Status and Doctor.
- M8 adds git Skill Sources as another Skill Source Adapter feeding the same
  prepared skill Interface.

Add one explicit task before filesystem mutation: define operation records and
Apply/Prune preconditions.

## Risks

- `operations` may become shallow generic machinery. Keep it policy-rich and
  grounded in Installed Artifact and Discovery Requirement facts.
- `skill_source` may grow too wide. Keep acquisition, Skill Groups, Skill
  Selection, Skill Alias, and git behavior in focused submodules.
- Preview may accidentally write while preparing hashes. Keep disk
  materialization only in Apply.
- Discovery Name preparation needs an exact frontmatter contract before
  implementation.
- Shared `.agents/skills` paths can hide collision bugs. Test shared Discovery
  Requirements across Codex, Pi, OpenCode, and Cursor together.

## Tests

Prioritize tests at each deep Module Interface:

- Skill Source discovery, Skill Selection, Skill Groups, Skill Alias, and
  Discovery Name output.
- Safe materialization, external symlink rejection, special-file rejection, and
  deterministic hashes.
- Lockfile round trips and plain Apply from Locked Desired State.
- Client Discovery Registry resolution, `clients = "all"`, and `--client`
  narrowing.
- Desired State assembly with shared Client Discovery Locations.
- Discovery Name Collision per Client Discovery Location.
- Apply refusal for Unmanaged Artifact and Unexpected Symlink Target.
- Prune removes only manifest-owned Stale Installed Artifacts and Stale
  Discovery Requirements.
- Preview read-only tests that snapshot config, Lockfiles, Manifest, Managed
  State, Skill Sources, and Client Discovery Locations before and after.
