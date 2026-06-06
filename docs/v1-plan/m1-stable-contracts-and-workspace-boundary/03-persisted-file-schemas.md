# 03: Persisted File Schemas

## Goal

Define persisted schema structs for ConfigDocs, Lockfiles, and the Manifest without implementing parsing or persistence.

## Read First

- `docs/v1-plan/m1-stable-contracts-and-workspace-boundary/README.md`
- `docs/design-v1.md` sections: "Config File Shape", "Manifest Model", and "Managed Skill Content"
- Task 01 output for shared IDs, `Client`, `ConfigLayerKind`, `InstallLevel`, and `TreeDigest`

## Scope

- Add serde derives for persisted schema structs.
- Use TOML kebab-case through serde renames.
- Define the `ConfigDoc` schema around the design-v1 shape:

```toml
version = 1
config-layer = "user-project"
clients = ["codex", "cursor"]

[[skills.sources]]
id = "team"
path = "../skills"
include = ["review"]
groups = ["rust"]
aliases = { review = "project-review" }
```

- Keep persisted schema types separate from normalized request types. The
  schema should mirror user-authored TOML; `config::build_request` later owns
  validation, `clients = "all"` expansion, and CLI narrowing.
- Keep persisted structs in the modules that own each file contract:
  TOML-facing config structs in `config/` and `config/skills.rs`, lockfile
  structs in `lockfile.rs`, and Manifest structs in `manifest.rs`.
- Include `config-layer` in `ConfigDoc` even though loaded config metadata also
  carries the expected Config Layer identity. Validation later rejects
  mismatches.
- Include `version = 1` as the same top-level `version` field on every
  persisted root document: `ConfigDoc`, `LockfileFile`, and `ManifestFile`.
- Define a plain `u32` schema-version constant for the initial persisted schema
  version. Do not add a version type or migration enum in this task.
- Represent omitted `clients` as missing, not as `all` or an empty list.
- Define a small `PersistedClientSelection` shape that accepts only
  `clients = "all"` or an explicit supported-client list such as
  `clients = ["codex", "cursor"]`.
- Define flat Skill Source config fields:
  - `id`
  - `path`
  - `git`
  - `rev`
  - `include`
  - `groups`
  - `aliases`
- Define skeletal lockfile containers with a top-level `version` field.
- Define close-to-final Manifest record structs:
  - `ManifestFile`
  - `InstalledArtifactRecord`
  - `DiscoveryRequirementRecord`
  - `ArtifactKey`
  - `RequirementKey`
  - `PinnedSkillRef`

## Implementation Notes

- Do not add the `toml` crate in M1.
- Do not implement read/write stores in this task.
- Apply serde derives only to persisted schema structs and shared value types that cross persisted boundaries.
- Do not derive `Default` for persisted root documents. They have required
  fields and a derived default would produce invalid files.
- Custom serde is allowed only for `PersistedClientSelection`, so the code can
  keep a direct `All` variant without adding serde-shape wrapper types.
- Omitted `include`, `groups`, and `aliases` default to empty.
- Store `aliases` as a deterministic map from `SourceSkillName` to
  `DiscoveryName`.
- Validation later enforces exactly one of `path` or `git`.
- Lockfile structs should contain top-level containers and terse comments describing what resolution fills in later.
- Persisted root document versions are schema fields only in this task; do not
  implement migration behavior.
- Manifest uses list records rather than encoded TOML map keys in M1.
- Manifest identity remains structured; do not make policy code depend on encoded key strings.

## Out Of Scope

- Custom serde implementations beyond the narrow `PersistedClientSelection`
  exception.
- TOML key escaping or map-key encoding.
- Config validation.
- Lockfile source-resolution metadata.
- Manifest store behavior.

## Acceptance Criteria

- Persisted schema structs express the user-facing TOML field names.
- Config schema preserves `config-layer`, `include`, `groups`, and `aliases`.
- Config, lockfile, and Manifest root schemas all include `version`.
- Manifest schema separates Installed Artifacts from Discovery Requirements.
- Lockfile schema avoids guessing M2/M3 source-resolution details.

## Validation

```bash
cargo check --workspace
```

Run `scripts/validate-all.sh` before completing M1.
