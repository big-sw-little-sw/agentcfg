# Alt-2 Design: Layer Contract Compiler

This is a concrete design option for V1. It expands the Alt-2 summary from
[architecture-alternatives-v1.md](architecture-alternatives-v1.md).

Alt-2 is not the recommended V1 path, but it is a coherent clean-slate design
if Config Layer and Lockfile correctness should dominate implementation shape.

## Thesis

Alt-2 makes Active Config Layers plus adjacent Lockfiles the deepest Module.
Its central Interface is:

```rust
compile_layer_contract(context, request) -> Result<LayerContract>
```

The Implementation hides Config Layer ordering, Lockfile reuse, Source Refresh,
Skill Selection, Skill Alias application, Managed Skill Content intent, and
Lockfile write intent. Client Discovery Locations, Desired State, Installed
Artifacts, Preview, Apply, Prune, Status, and Doctor are projections from that
compiled contract.

```text
Active Config Layers + Lockfiles
  -> LayerContract
  -> Client Discovery Location projection
  -> Desired State projection
  -> install health
  -> operation projection
```

"LayerContract" is internal architecture language. User-facing and persisted
language remains Config Layer, Desired State, Locked Desired State, Lockfile,
Manifest, Discovery Requirement, and Installed Artifact.

## Module Layout

```text
crates/agentcfg-core/src/
  workflow/
    context.rs
    install_context.rs
    preview.rs
    apply.rs
    prune.rs
    status.rs
    doctor.rs
    types.rs

  layer_contract/
    mod.rs
    records.rs
    compile.rs
    lock_reconcile.rs
    skill_adapter.rs
    diagnostics.rs

  discovery_registry/
    mod.rs
    resolution.rs

  desired_state/
    mod.rs
    projection.rs
    records.rs

  install_health/
    mod.rs

  operations/
    mod.rs
    records.rs
    preview.rs
    apply.rs
    prune.rs

  skill_content/
    mod.rs
    hashing.rs

  lockfile.rs
  manifest.rs
  config.rs
  config_paths.rs
  skill_source/
```

The `workflow` Module remains an Adapter. The Layer Contract compiler owns the
deeper policy. Existing `config`, `config_paths`, and `skill_source` Modules
stay focused.

## Module Interfaces

```rust
struct LayerContractRequest {
    install_level: InstallLevel,
    source_resolution_policy: SkillSourceResolutionPolicy,
}

struct ActiveInstallContext {
    install_level: InstallLevel,
    active_layers: Vec<ActiveLayerPaths>,
    managed_state_paths: ManagedStatePaths,
    project_root: Option<PathBuf>,
    user_dirs: UserDirs,
}

fn compile_layer_contract(
    context: &ActiveInstallContext,
    request: LayerContractRequest,
) -> Result<LayerContract>;
```

```rust
struct LayerContract {
    install_level: InstallLevel,
    layers: Vec<CompiledConfigLayer>,
    skills: Vec<ContractSkill>,
    client_selections: Vec<ContractClientSelection>,
    lockfile_changes: Vec<LockfileChange>,
    warnings: Vec<ContractWarning>,
}

struct ContractSkill {
    config_layer: ConfigLayer,
    skill_source_id: String,
    source_skill_name: String,
    discovery_name: String,
    source_hash: Hash,
    installed_hash: Hash,
    managed_skill_content_path: PathBuf,
    discovery_name_prepared: bool,
    acquisition_mode: SkillSourceAcquisitionMode,
    content_intent: ManagedSkillContentIntent,
}

enum ManagedSkillContentIntent {
    AlreadyPresent,
    NeedsMaterialization(MaterializedSkillTree),
    MissingAndRebuildable(MaterializedSkillTree),
    MissingAndBlocked(MissingManagedSkillContentReason),
}
```

The main private Seam is `layer_contract::skill_adapter`. In V1 there is one
Adapter, Skill. The Seam stays private because a public generic Configured Item
Interface would be speculative.

## Projection Records

```rust
struct ClientProjection {
    locations: Vec<ResolvedClientDiscoveryLocation>,
    warnings: Vec<ClientDiscoveryWarning>,
}

struct DesiredStateProjection {
    desired_installed_artifacts: Vec<DesiredInstalledArtifact>,
    discovery_name_collisions: Vec<DiscoveryNameCollision>,
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
```

The contract does not perform filesystem mutation. It produces enough facts for
later projections to be deterministic.

## Workflow Flow

### Preview

```text
PreviewRequest
  -> ActiveInstallContext
  -> LayerContract
  -> ClientProjection
  -> DesiredStateProjection
  -> Manifest snapshot
  -> InstallHealth
  -> Preview operation projection
  -> PreviewResult
```

Preview may compute refreshed lock facts in memory, but writes nothing.

### Apply

```text
same compile/projection path as Preview
  -> Apply operation projection
  -> write Lockfiles only when needed
  -> materialize Managed Skill Content
  -> create or update manifest-owned Installed Artifacts
  -> add Discovery Requirements
  -> write Manifest
```

Apply never removes Stale Installed Artifacts or Stale Discovery Requirements.

### Prune

```text
PruneRequest
  -> ActiveInstallContext
  -> LayerContract using UseLocked
  -> DesiredStateProjection where provable
  -> Manifest snapshot
  -> InstallHealth
  -> Prune operation projection
  -> remove stale manifest-owned state
```

If Desired State cannot be proven for a selected all-skills Skill Source because
source and lock facts are insufficient, Prune refuses affected removals.

### Status

Status uses `LayerContract`, Desired State projection, Manifest, and filesystem
facts to report install-state consistency. It does not duplicate Doctor
readiness checks except where a blocker affects Status itself.

### Doctor

Doctor checks readiness: config validity, Project Root detection, git
availability, supported Clients, path writability, Client Discovery Location
confidence, and optional Skill Source reachability. It does not replace Status.

## Persistence

Lockfile records remain adjacent to Config Layers.

```toml
[[skills]]
skill_source_id = "personal"
skill_source_type = "path"
source_skill_name = "do-code-review"
discovery_name = "code-review"
source_hash = "sha256:..."
installed_hash = "sha256:..."
discovery_name_prepared = true
acquisition_mode = "copy"
materialization_version = 1
```

Manifest records stay under Managed State and record ownership at Client
Discovery Locations.

```json
{
  "configured_item_kind": "skill",
  "skill_source_id": "personal",
  "source_skill_name": "do-code-review",
  "discovery_name": "code-review",
  "client_discovery_location": ".agents/skills",
  "installed_artifact_path": ".agents/skills/code-review",
  "expected_symlink_destination": ".agentcfg/skills/sha256/...",
  "installed_hash": "sha256:...",
  "discovery_requirements": [
    {
      "config_layer": "shared-project",
      "client": "codex",
      "install_level": "project"
    }
  ]
}
```

## Safety Invariants

- Config Layer, Install Level, and Persisted Scope Value never collapse into one
  field.
- Source Refresh is the only path that changes lock facts.
- Lockfile write intent is computed before filesystem mutation.
- Discovery Name Collision is detected after Client Discovery Location
  projection and before mutation.
- Shared Client Discovery Locations produce one Installed Artifact with many
  Discovery Requirements.
- Apply only creates or updates manifest-owned Installed Artifacts and refuses
  Unmanaged Artifacts.
- Apply and Prune refuse Unexpected Symlink Target.
- Prune removes only manifest-owned stale state.
- External symlinks and special files in Skill Sources are rejected during
  materialization.
- Deterministic ordering is required for LayerContract records, Lockfiles,
  Manifest records, and operation records.

## Tests

The compiler Interface becomes the main test surface.

- Project Level layer order: Shared Project Config then User Project Config.
- User Level uses User Config only.
- Missing Lockfile and stale Lockfile facts.
- Source Refresh computes refreshed facts in memory during Preview.
- Deterministic LayerContract order.
- Skill Alias changes Discovery Name while preserving Source Skill Name.
- Missing Managed Skill Content: rebuildable versus blocked cases.
- `clients = "all"`, explicit Clients, repeated `--client`, and invalid Client
  filters.
- Desired State projection merges Discovery Requirements for shared Client
  Discovery Locations.
- Discovery Name Collision per Client Discovery Location.
- Apply refuses Unmanaged Artifact and Unexpected Symlink Target.
- Prune removes Stale Discovery Requirements and Stale Installed Artifacts only
  when safe.
- Status and Doctor consume the right facts for their distinct questions.

## Post-V1 Posture

Alt-2 keeps future extension at the Layer Contract compiler Seam without making
that Seam public in V1. A future MCP-specific Adapter could compile MCP facts
from Active Config Layers and Lockfiles, while owning MCP validation, secrets
posture, and native file safety.

Do not force future Configured Item kinds into the Skill Installed Artifact
model. The Layer Contract compiler should remain internal until a second kind
proves the Interface.

## When To Choose This

Choose Alt-2 if the highest risk is persisted contract correctness: Config
Layer composition, Lockfile drift, Source Refresh, and Manifest ownership.

Do not choose Alt-2 if near-term delivery is the priority. It moves more work
earlier than Alt-1 before Preview and Apply can ship.
