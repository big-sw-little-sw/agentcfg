# Alt-1 Design: Skill-First Resolution, Shared Operations

This is a concrete design option for V1. It expands the Alt-1 summary from
[architecture-alternatives-v1.md](architecture-alternatives-v1.md).

Alt-1 is also the recommended direction, with the refinements captured in
[architecture-recommendation-v1.md](architecture-recommendation-v1.md).

## Thesis

V1 keeps the deepest Module skill-first. Skill Configuration, Skill Source
resolution, Skill Selection, Skill Alias handling, Managed Skill Content
preparation, and hashing stay in skill-specific Modules.

Shared operations begin only after those Modules emit structured Desired State
records for Installed Artifacts.

```text
Skill Configuration
  -> Skill Source resolution
  -> Skill Selection
  -> Managed Skill Content facts
  -> Desired State
  -> shared operation planning
  -> Preview / Apply / Prune / Status / Doctor
```

The main Seam is between skill-specific resolution and shared operation
planning. V1 has one Configured Item kind, Skill, so there is no generic
Configured Item manager Interface.

## Module Layout

```text
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

  discovery_registry/
    mod.rs
    resolution.rs

  desired_state/
    mod.rs
    skill.rs
    records.rs

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

  lockfile.rs
  manifest.rs
```

Names are illustrative. The important design is the ownership split.

## Module Interfaces

| Module | Interface | Implementation ownership |
| --- | --- | --- |
| `workflow` | public workflow request/result functions | Thin orchestration from CLI-facing requests to deeper Modules. |
| `workflow::install_context` | `build_install_context(request)` | Install Level, Active Config Layers, Project Root, User paths, config paths, Lockfile paths, Managed State roots. |
| `config` | load and validate Skill Configuration | TOML parsing, Persisted Scope Value checks, skill-specific config shape. |
| `skill_source` | `resolve_skill_selection(inputs, policy)` | Path/git acquisition, Skill Source discovery, Included Skills, Skill Groups, Skill Aliases, Discovery Name output. |
| `skill_content` | `prepare_skill_content(selected_skill)` | Safe tree walk, Discovery Name preparation, symlink and special-file policy, deterministic hashes. |
| `managed_skill_content` | compute/read/write Managed Skill Content | Content-addressed Managed Skill Content under Managed State. |
| `discovery_registry` | `resolve_client_discovery_locations(...)` | Client catalog, `clients = "all"`, `--client` narrowing, Client Discovery Locations, confidence/provenance. |
| `desired_state` | `build_desired_state(skill_facts, client_facts)` | Desired Installed Artifacts and Discovery Requirements. |
| `manifest` | read/write Manifest records | Installed Artifact ownership and Discovery Requirement persistence. |
| `install_health` | `classify_install_state(desired_state, manifest)` | Broken Symlink, Unexpected Symlink Target, Unmanaged Artifact, stale and unsatisfied facts. |
| `operations` | `build_operation_plan(...)` | Discovery Name Collision, creates, updates, stale reports, prune candidates, warnings. |
| `execution` | `apply_operation_plan`, `prune_operation_plan` | Filesystem writes with precondition checks. |

## Core Records

Records should use glossary terms in field names wherever practical.

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
struct InstallHealth {
    unmanaged_artifacts: Vec<UnmanagedArtifact>,
    broken_symlinks: Vec<BrokenSymlink>,
    unexpected_symlink_targets: Vec<UnexpectedSymlinkTarget>,
    missing_managed_skill_content: Vec<MissingManagedSkillContent>,
    stale_discovery_requirements: Vec<StaleDiscoveryRequirement>,
    unsatisfied_discovery_requirements: Vec<UnsatisfiedDiscoveryRequirement>,
    stale_installed_artifacts: Vec<StaleInstalledArtifact>,
    unused_managed_skill_content: Vec<UnusedManagedSkillContent>,
}

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

## Workflow Flow

### Preview

```text
PreviewRequest
  -> InstallLevelContext
  -> load Active Config Layers
  -> read adjacent Lockfiles
  -> resolve Skill Sources with UseLocked or in-memory Source Refresh
  -> apply Skill Selection and Skill Aliases
  -> prepare Managed Skill Content facts in memory
  -> resolve Clients and Client Discovery Locations
  -> build Desired State
  -> read Manifest and classify InstallHealth
  -> build OperationPlan
  -> return PreviewResult
```

Preview does not receive a write-capable Adapter. It cannot write config,
Lockfiles, Manifest, Managed State, Skill Sources, or Client Discovery Locations.

### Apply

```text
same fact-building path as Preview
  -> reject Discovery Name Collision and unsafe install health
  -> write missing or refreshed Lockfiles
  -> materialize Managed Skill Content
  -> create or update manifest-owned Installed Artifacts
  -> add Discovery Requirements
  -> write Manifest
  -> warn about stale Managed State
```

Apply does not remove Stale Discovery Requirements or Stale Installed Artifacts.

### Prune

```text
PruneRequest
  -> InstallLevelContext
  -> build Desired State from active config and Locked Desired State
  -> read Manifest and classify InstallHealth
  -> remove Stale Discovery Requirements
  -> remove Stale Installed Artifacts only when no Discovery Requirements remain
  -> remove unused Managed Skill Content only when safe
```

Prune acts from Manifest ownership plus current filesystem checks. It does not
delete Unmanaged Artifacts.

### Status

Status builds Desired State, reads Manifest, inspects Managed State and Client
Discovery Locations, then reports managed install-state consistency.

It reports Installed Artifacts, Broken Symlinks, Unexpected Symlink Targets,
missing Managed Skill Content, Stale Installed Artifacts, Unsatisfied Discovery
Requirements, config/Lockfile mismatch, Manifest readability, and informational
Unmanaged Artifacts.

### Doctor

Doctor checks readiness, not install-state consistency. It validates config,
Project Root detection, git availability, supported Clients, path writability,
Client Discovery Location confidence, and optional Skill Source reachability.

## Persistence

Lockfiles are adjacent to Config Layers and record Locked Desired State only.
They do not store machine-local Client Discovery Location paths.

```toml
[[skills]]
skill_source_id = "personal"
skill_source_type = "path"
source_skill_name = "do-code-review"
discovery_name = "code-review"
source_hash = "sha256:..."
installed_hash = "sha256:..."
discovery_name_prepared = true
materialization_version = 1
```

The Manifest is under Managed State and records ownership, not Desired State.

```json
{
  "configured_item_kind": "skill",
  "client_discovery_location": ".agents/skills",
  "discovery_name": "code-review",
  "installed_artifact_path": ".agents/skills/code-review",
  "expected_symlink_destination": ".agentcfg/skills/sha256/...",
  "installed_hash": "sha256:...",
  "skill_source_id": "personal",
  "source_skill_name": "do-code-review",
  "discovery_requirements": [
    {
      "config_layer": "user-project",
      "client": "codex",
      "install_level": "project"
    }
  ]
}
```

## Safety Invariants

- Preview is read-only by construction.
- Apply is one-way from Skill Source to Managed Skill Content to Client
  Discovery Location.
- Apply never writes Skill Sources.
- Apply never removes stale Managed State.
- Prune is the only workflow that removes Stale Discovery Requirements or
  Stale Installed Artifacts.
- Unmanaged Artifacts are never overwritten or deleted.
- Unexpected Symlink Target blocks Apply and Prune.
- External symlinks and special files in Skill Sources are rejected during
  materialization.
- Discovery Name Collision is checked per Client Discovery Location before
  mutation.
- Discovery Requirements are keyed by Config Layer, Client, and Install Level.
- Shared Client Discovery Locations use one Installed Artifact with many
  Discovery Requirements.

## Tests

Focus tests at deep Module Interfaces.

- Skill Source discovery, Skill Selection, Skill Groups, Skill Aliases, and
  Discovery Name output.
- Safe materialization, external symlink rejection, special-file rejection, and
  deterministic hashes.
- Preview read-only with and without Source Refresh.
- Plain Apply from Locked Desired State.
- Source Refresh updates Lockfile and Managed Skill Content.
- Client Discovery Registry resolution, `clients = "all"`, and `--client`
  narrowing.
- Desired State assembly with shared Client Discovery Locations.
- Discovery Name Collision per Client Discovery Location.
- Apply refusal for Unmanaged Artifact and Unexpected Symlink Target.
- Prune removes only manifest-owned stale state.
- Status and Doctor answer distinct questions.

## Post-V1 Posture

Future Configured Item kinds should add kind-specific resolvers that emit Desired
State records. Shared operations can then reuse Manifest ownership, install
health, Preview, Apply, Prune, and Status behavior.

V1 reserves:

- `ConfiguredItemKind::Skill` in Desired State, Manifest, operation records, and
  Client Discovery Registry keys
- aggregate command meaning, where `agentcfg apply` applies Agent Configuration
  for the selected Install Level
- kind-aware operation records

V1 defers:

- generic Configured Item resolver traits
- generic Configured Item config schema
- dynamic registration of Configured Item kinds
- cross-kind transaction semantics
- MCP, rules, and agent-definition conflict rules

## Adoption Notes

Adopt this incrementally:

1. Add `workflow::install_context`.
2. Deepen `discovery_registry` so it owns Client resolution and `--client`
   narrowing.
3. Split Skill Alias handling out of broad Skill Selection code.
4. Add `skill_content` before Lockfile writes depend on hashes.
5. Build `desired_state` as fan-in from prepared skill facts and resolved
   Client facts.
6. Add `operations` before filesystem mutation.
7. Keep terminal rendering in `agentcfg-cli`.
