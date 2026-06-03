# Alt-3 Design: Workflow Kernel

This is a concrete design option for V1. It expands the Alt-3 summary from
[architecture-alternatives-v1.md](architecture-alternatives-v1.md).

Alt-3 is not the recommended V1 path. Its useful contribution is explicit
workflow transition safety, which the recommendation borrows in a smaller form.

## Thesis

Preview, Apply, Prune, Status, and Doctor are modeled as allowed transitions
over one install-state machine. The deep Module is `workflow_kernel`: callers
ask for workflow outcomes while phase ordering, safety checks, Desired State
comparison, Manifest interpretation, and commit rules stay inside one
Implementation.

```text
workflow request
  -> install-state machine
  -> transition set
  -> optional commit
```

Choose this if the main V1 risk is workflow drift: Preview says one thing,
Apply does another, Prune uses different facts, and Status reports a different
model.

## Module Layout

```text
crates/agentcfg-core/src/
  workflow/
    mod.rs
    types.rs
    init.rs
    context.rs

  workflow_kernel/
    mod.rs
    session.rs
    skill_intent.rs
    locked_state.rs
    managed_content.rs
    discovery.rs
    snapshot.rs
    transition.rs
    commit.rs
    doctor.rs

  config.rs
  config_paths.rs
  skill_source/
  skill_content/
  desired_state.rs
  lockfile.rs
  manifest.rs
  install_health.rs
  discovery_registry.rs
```

`workflow` stays a public Adapter. `workflow_kernel` owns the transition graph.
Existing Modules become leaf Modules used by internal phases.

## Public Workflow Interface

```rust
pub fn preview(request: PreviewRequest) -> Result<PreviewResult>;
pub fn apply(request: ApplyRequest) -> Result<ApplyResult>;
pub fn prune(request: PruneRequest) -> Result<PruneResult>;
pub fn status(request: StatusRequest) -> Result<StatusResult>;
pub fn doctor(request: DoctorRequest) -> Result<DoctorResult>;
```

## Internal Kernel Interfaces

```rust
fn build_install_snapshot(
    request: KernelRequest,
    reader: &impl InstallStateRead,
) -> Result<InstallSnapshot>;

fn build_transition_set(snapshot: &InstallSnapshot) -> Result<TransitionSet>;

fn commit_apply(
    snapshot: &InstallSnapshot,
    transitions: &TransitionSet,
    writer: &impl InstallStateWrite,
) -> Result<ApplyOutcome>;

fn commit_prune(
    snapshot: &InstallSnapshot,
    transitions: &TransitionSet,
    writer: &impl InstallStateWrite,
) -> Result<PruneOutcome>;
```

Filesystem and git are real Seams because production behavior and tests use
different Adapters. Generic Configured Item resolver Interfaces are not added in
V1 because Skill is the only Configured Item kind.

```rust
trait InstallStateRead {
    fn read_config(...);
    fn read_lockfile(...);
    fn read_manifest(...);
    fn inspect_client_discovery_location(...);
    fn read_managed_skill_content(...);
}

trait InstallStateWrite {
    fn write_lockfile(...);
    fn write_managed_skill_content(...);
    fn create_or_update_symlink_if_safe(...);
    fn write_manifest(...);
    fn remove_installed_artifact_if_safe(...);
}
```

## Kernel Records

```rust
struct InstallSession {
    install_level: InstallLevel,
    active_config_layers: Vec<ConfigLayer>,
    config_paths: ConfigFilePaths,
    lockfile_paths: LockfilePaths,
    managed_state_paths: ManagedStatePaths,
    project_root: Option<PathBuf>,
    user_dirs: UserDirs,
}

struct ResolvedSkillIntent {
    config_layer: ConfigLayer,
    skill_source_id: String,
    source_skill_name: String,
    discovery_name: String,
    skill_directory: PathBuf,
    discovery_name_preparation_needed: bool,
}

struct ManagedContentIntent {
    managed_skill_content_path: PathBuf,
    source_hash: Hash,
    installed_hash: Hash,
    acquisition_mode: SkillSourceAcquisitionMode,
    content_state: ManagedContentState,
}

struct InstallSnapshot {
    session: InstallSession,
    locked_skills: Vec<LockedSkill>,
    desired_installed_artifacts: Vec<DesiredInstalledArtifact>,
    manifest_installed_artifacts: Vec<ManifestInstalledArtifact>,
    filesystem_facts: Vec<FilesystemFact>,
    install_health: InstallHealth,
}

struct TransitionSet {
    lockfile_transitions: Vec<LockfileTransition>,
    managed_skill_content_transitions: Vec<ManagedSkillContentTransition>,
    installed_artifact_transitions: Vec<InstalledArtifactTransition>,
    discovery_requirement_transitions: Vec<DiscoveryRequirementTransition>,
    stale_reports: Vec<StaleReport>,
    warnings: Vec<WorkflowWarning>,
}
```

`InstallSnapshot` and `TransitionSet` are ephemeral. They are never persisted.

## Workflow Flow

### Preview

```text
PreviewRequest
  -> InstallSession
  -> load Active Config Layers, Lockfiles, Manifest
  -> resolve Skill Selection and Discovery Names
  -> reconcile Desired State and Locked Desired State in memory
  -> resolve Clients and Client Discovery Locations
  -> build InstallSnapshot
  -> build TransitionSet
  -> return PreviewResult
```

Preview receives only `InstallStateRead`. It cannot write by construction.

### Apply

```text
same snapshot path as Preview
  -> commit apply transitions only
  -> write missing or refreshed Lockfiles when allowed
  -> write Managed Skill Content
  -> create or update manifest-owned Installed Artifacts
  -> add Discovery Requirements
  -> write Manifest
  -> return stale warnings
```

Apply never removes Stale Installed Artifacts or Stale Discovery Requirements.

### Prune

```text
PruneRequest
  -> InstallSession
  -> load current config, Lockfiles, Manifest, filesystem facts
  -> build InstallSnapshot
  -> build prune transitions
  -> remove Stale Discovery Requirements
  -> remove Stale Installed Artifacts only when no Discovery Requirements remain
  -> remove unused Managed Skill Content only when safe
```

Prune is not Apply with deletes enabled. It is a separate transition over
Manifest-owned state.

### Status

Status builds an `InstallSnapshot` and reports managed install-state
consistency: Installed Artifacts, Broken Symlinks, Unexpected Symlink Targets,
missing Managed Skill Content, Stale Installed Artifacts, Unsatisfied Discovery
Requirements, config/Lockfile mismatch, Manifest readability, and informational
Unmanaged Artifacts.

### Doctor

Doctor checks environment and configuration readiness: Project Root detection,
config schema validity, git availability, supported Clients, path writability,
Client Discovery Location confidence, and optional Skill Source reachability.

Doctor does not replace Status.

## Safety Invariants

- Preview is read-only by construction.
- Apply is one-way from Skill Source to Managed Skill Content to Client
  Discovery Location.
- Apply never writes Skill Sources.
- Prune is the only workflow that removes stale Managed State.
- Lockfile is the repeatability authority for Locked Desired State.
- Manifest is the ownership authority for Installed Artifacts and Discovery
  Requirements.
- Client Discovery Registry is the location authority for Client Discovery
  Locations.
- Discovery Name Collision is detected per Client Discovery Location before
  commit.
- Unmanaged Artifacts are never overwritten or deleted.
- Unexpected Symlink Target blocks update and prune.
- Commit rechecks every filesystem precondition immediately before writing.
- Config Layer, Install Level, and Persisted Scope Value remain distinct.

## Persistence

Lockfiles stay adjacent to each Config Layer and record Locked Desired State
only. They must not store machine-local Client Discovery Location paths.

Manifest records stay under Managed State and record physical ownership:
Installed Artifact path, expected symlink destination, installed hash, Skill
Source provenance, and Discovery Requirements.

The kernel records are runtime facts only.

## Tests

Tests focus on the kernel Interface because the Interface is the test surface.

- Preview read-only with and without Source Refresh.
- Plain Apply uses Locked Desired State.
- Apply with Source Refresh updates Lockfile and Managed Skill Content.
- Discovery Name Collision fails per Client Discovery Location.
- Shared `.agents/skills` paths merge Discovery Requirements for Codex, Pi,
  OpenCode, and Cursor.
- Unexpected Symlink Target blocks Apply.
- Unexpected Symlink Target blocks Prune.
- Prune removes only manifest-owned stale state.
- Status reports install health from the same snapshot shape used by Apply and
  Prune.
- Doctor does not report install-state consistency.
- Commit preconditions are rechecked immediately before writes.

## Post-V1 Posture

Alt-3 gives future Leverage when several Configured Item kinds exist and Apply
may need native config-file edits instead of only symlinked Installed Artifacts.

V1 should still defer:

- generic resolver/applier/status Adapter Interfaces
- dynamic registration of Configured Item kinds
- cross-kind transaction semantics
- rollback semantics for native config-file edits
- generic external-origin language beyond Skill Source

## When To Choose This

Choose Alt-3 if workflow safety drift is the dominant risk and the team accepts
a wider core Module.

Do not choose Alt-3 if near-term V1 delivery is the priority. Its small public
Interface hides substantial phase coupling, and V1 has only one Configured Item
kind to justify it.
