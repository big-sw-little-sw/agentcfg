# 03: Persisted File Schemas

## Goal

Define persisted schema structs for Config Layer files, Lockfiles, and the Manifest without implementing parsing or persistence.

## Read First

- `docs/v1-plan/m1-stable-contracts-and-workspace-boundary/README.md`
- `docs/design-v1.md` sections: "Config File Shape", "Manifest Model", and "Managed Skill Content"
- Task 01 output for shared IDs, `Client`, `ConfigLayerKind`, `InstallLevel`, and `TreeDigest`

## Scope

- Add serde derives for persisted schema structs.
- Use TOML kebab-case through serde renames.
- Define Config Layer schema around the design-v1 shape:

```toml
config-layer = "user-project"
clients = ["codex", "cursor"]

[[skills.sources]]
id = "team"
path = "../skills"
include = ["review"]
groups = ["rust"]
aliases = { review = "project-review" }
```

- Define flat Skill Source config fields:
  - `id`
  - `path`
  - `git`
  - `rev`
  - `include`
  - `groups`
  - `aliases`
- Define skeletal lockfile containers.
- Define close-to-final Manifest record structs:
  - `ManifestFile`
  - `InstalledArtifactRecord`
  - `DiscoveryRequirementRecord`
  - `ArtifactKey`
  - `RequirementKey`
  - `LockedSkillRef`

## Implementation Notes

- Do not add the `toml` crate in M1.
- Do not implement read/write stores in this task.
- Apply serde derives only to persisted schema structs and shared value types that cross persisted boundaries.
- Omitted `include`, `groups`, and `aliases` default to empty.
- Validation later enforces exactly one of `path` or `git`.
- Lockfile structs should contain top-level containers and terse comments describing what lock planning fills in later.
- Manifest uses list records rather than encoded TOML map keys in M1.
- Manifest identity remains structured; do not make policy code depend on encoded key strings.

## Out Of Scope

- Custom serde implementations.
- TOML key escaping or map-key encoding.
- Config validation.
- Lockfile source-resolution metadata.
- Manifest store behavior.

## Acceptance Criteria

- Persisted schema structs express the user-facing TOML field names.
- Config schema preserves `config-layer`, `include`, `groups`, and `aliases`.
- Manifest schema separates Installed Artifacts from Discovery Requirements.
- Lockfile schema avoids guessing M2/M3 source-resolution details.

## Validation

```bash
cargo check --workspace
```

Run `scripts/validate-all.sh` before completing M1.
