# agentcfg V1 Design

This document defines V1 persisted contracts and safety rules. Product behavior and user-facing workflows are summarized in [prd.md](prd.md).

## Files and State

Project and CLI name:

```text
agentcfg
```

Shared project config:

```text
agentcfg.toml
agentcfg.lock
```

User project config:

```text
.agentcfg/config.toml
.agentcfg/lock.toml
```

User config:

```text
${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml
${XDG_CONFIG_HOME:-~/.config}/agentcfg/lock.toml
```

Generated project state:

```text
.agentcfg/manifest.json
.agentcfg/sources/
```

Generated user state:

```text
${XDG_STATE_HOME:-~/.local/state}/agentcfg/manifest.json
${XDG_STATE_HOME:-~/.local/state}/agentcfg/sources/
```

`.agentcfg/` should be ignored by default. `agentcfg.lock` is committed only when a project intentionally adopts shared agent config.

## Config Layers and Install Levels

V1 exposes three Config Layers:

| Config Layer | Persisted Scope Value | Core name | Meaning |
| --- | --- | --- | --- |
| User Config | `user` | `ConfigLayer::User` | Current user's home/config layer. |
| Shared Project Config | `shared-project` | `ConfigLayer::SharedProject` | Shared repo-level config. |
| User Project Config | `user-project` | `ConfigLayer::UserProject` | Current user's config for one repo. |

Active Config Layers by command:

| Command | Active Config Layers | Install level |
| --- | --- | --- |
| `agentcfg preview` | Shared Project Config, then User Project Config | project level |
| `agentcfg preview --refresh-sources` | Shared Project Config, then User Project Config | project level |
| `agentcfg apply` | Shared Project Config, then User Project Config | project level |
| `agentcfg apply --refresh-sources` | Shared Project Config, then User Project Config | project level |
| `agentcfg prune` | manifest Discovery Requirements for project Client Discovery Locations | project level |
| `agentcfg status` | Shared Project Config, User Project Config, project manifest | project level |
| `agentcfg doctor` | all discoverable config and environment state | diagnostic only |
| `agentcfg preview --user` | User Config only | user level |
| `agentcfg preview --user --refresh-sources` | User Config only | user level |
| `agentcfg apply --user` | User Config only | user level |
| `agentcfg apply --user --refresh-sources` | User Config only | user level |
| `agentcfg prune --user` | stale Discovery Requirements and Installed Artifacts in user Client Discovery Locations | user level |
| `agentcfg status --user` | User Config and user manifest | user level |

Layer order:

```text
Shared Project Config -> User Project Config
```

User project config is additive by default. It may add sources, selected skills, aliases, and clients for the current user in the current repo. It must not silently replace or weaken shared project config. Explicit override semantics are out of V1.

Source ids are namespaced by layer internally:

```text
{scope}:{source_id}
```

This allows shared project config and user project config to both use a source id such as `local` without collision. User-facing diagnostics should include both the human source id and the layer when ambiguity matters.

Discovery Names are not namespaced. After Skill Alias resolution, Discovery Names must be unique per Client Discovery Location. If two active layers resolve to the same Discovery Name at the same Client Discovery Location:

- If they refer to the same locked Source Skill Name and same installed hash, merge Discovery Requirements.
- If they differ, fail with a **Discovery Name Collision** error and require a Skill Alias.

Aliases are applied before collision detection.

Client selection is additive across active layers. If shared project config selects `codex` and user project config selects `opencode`, the desired project install includes both clients. If CLI `--client` is omitted, commands at a given Install Level use the full configured client set for the active Config Layers. CLI `--client` may be repeated to narrow that configured client set, but must not add clients outside the configured selection in V1.

Discovery Requirements are keyed by Config Layer, Client, and Install Level (serialized forms may use Persisted Scope Value for the layer). Removing a skill or client from one layer makes that Discovery Requirement stale. `prune` removes stale Discovery Requirements and deletes the Installed Artifact only when no Discovery Requirements remain.

In code, use `ConfigLayer` for the config file/layer being initialized or loaded, `InstallLevel` for project-vs-user installation, and reserve low-level `target_path` / symlink **target** only for filesystem symlink diagnostics and internal manifest fields—not user-facing "client target" language.

Use `SourceResolutionPolicy::{UseLocked, RefreshSources}` for the core source-resolution choice so the workflow API expresses **Source Refresh** without leaking the CLI flag name `--refresh-sources`.

## Config Schema

V1 config is skill-specific. Do not expose or implement a generic resource schema in V1.

Example:

```toml
scope = "user-project"

[[skill_sources]]
id = "personal"
type = "path"
path = "../my-agent-skills/skills"
include = ["do-code-review"]
groups = ["design"]

[skill_aliases]
"personal:legacy-review" = "code-review"

[skills]
clients = ["codex", "claude", "opencode"]
```

To target every client supported by the current `agentcfg` version:

```toml
[skills]
clients = "all"
```

Rules:

- `scope` is required and must match the config location.
- `[[skill_sources]]` entries require explicit `id`.
- **Skill Selection** lives only under `[[skill_sources]]`.
- `include` is an optional list of **Included Skills** (Source Skill Names) from that Skill Source.
- `groups` is an optional list of **Skill Group** names from that Skill Source's `skills.toml`.
- If neither `include` nor `groups` is set, select all discovered skills from that Skill Source.
- `exclude` is out of V1.
- Missing `[skills].clients` is a validation error.
- `[skills].clients` may be either an explicit non-empty list of client ids or the string `"all"`.
- `clients = "all"` means every `agentcfg`-supported Client Discovery Location that is enabled for the selected Install Level in the current `agentcfg` version. It does not mean every agent application installed on the machine. Because this can expand when `agentcfg` adds support for new clients, `preview` must show the resolved client set before `apply` writes any new Client Discovery Location.
- `[skills]` owns install-wide skill behavior such as target clients. It does not select skill names in V1.
- CLI `--client` may narrow configured clients, but should not expand beyond configured clients in V1. If omitted, all configured clients remain selected. With `clients = "all"`, `--client` may narrow to any supported client.
- **Skill Aliases** live under `[skill_aliases]`, are local to the config layer that declares them, and use qualified `source_id:source_skill_name` keys (Source Skill Name).
- Skill Aliases are applied after source-local group expansion and before Discovery Name Collision detection.

Potential future config sections:

```toml
[[mcp_servers]]
[[hooks]]
[[rules]]
```

Future resource sections may share planner/apply concepts, but V1 should remain skill-first until a second resource kind exists.

## Source Discovery

Supported V1 source types:

- `path`: a filesystem directory containing skills
- `git`

Default behavior:

- Git Skill Sources are copied into the active scope's Managed Skill Content directory.
- Path Skill Sources are also copied by default.
- Direct source symlink mode is deferred from V1. This would mean pointing a Client Discovery Location directly at the original Skill Source directory instead of Managed Skill Content.

Source manifests:

- `skills.toml` inside a source is optional.
- Skills in **Agent Skill Format** can be inferred from directories containing `SKILL.md`.
- `skills.toml` may define source-local groups.

V1 `skills.toml` schema:

```toml
[groups]
design = [
  "do-design-pass",
  "do-design-review",
  "do-maintainability-check",
]
```

Groups are namespaced by source. Two sources may both define `design`.

## Artifact Model

V1 separates source acquisition from target installation.

Source acquisition mode:

- `copy`: default for path and git Skill Sources. The selected skill is materialized into the active scope's Managed Skill Content directory.

Managed Skill Content paths:

```text
project apply: .agentcfg/sources/<layer>/<source-id>/<resolved-id>/<discovery-name>/
user apply:    ${XDG_STATE_HOME:-~/.local/state}/agentcfg/sources/<layer>/<source-id>/<resolved-id>/<discovery-name>/
```

The exact path format can change during implementation, but it must be stable enough for manifests and diagnostics. Managed Skill Content is the canonical materialized content for copied sources.

Why Managed Skill Content is required:

- Plain `apply` can reinstall the locked skill version without rereading a mutable path Skill Source or floating git ref.
- `apply --refresh-sources` (Source Refresh) materializes new Managed Skill Content, updates the lockfile, and retargets Client Discovery Location symlinks.
- Old Managed Skill Content can remain as cache until no lockfile or manifest target uses it, then `prune` may remove it.

Installed Artifact mode:

- V1 default is `symlink` from the Client Discovery Location path to the managed materialized tree.
- V1 does not need user-configurable target mode (internal manifest field).
- A future version may add copied Client Discovery Location directories if a client proves incompatible with symlinked skills.

For copied sources:

```text
Skill Source -> Managed Skill Content -> Client Discovery Location symlink
```

Client Discovery Location symlinks must point to the managed materialized tree for the same apply scope:

- project Client Discovery Locations point into project generated state under `.agentcfg/sources/`
- user Client Discovery Locations point into user generated state under `${XDG_STATE_HOME:-~/.local/state}/agentcfg/sources/`

Expected symlink target validation:

- Manifest records the expected symlink target for each symlinked Installed Artifact.
- `status` reports missing links, broken targets, and unexpected symlink targets.
- `prune` refuses to remove a symlink whose current target does not match the manifest.
- `apply` may update a manifest-owned symlink when the previous target matches the manifest and the desired target changed.

Apply direction is one-way:

```text
Skill Source -> Managed Skill Content -> Client Discovery Location
```

`agentcfg` never writes changes back to a Skill Source. To improve a skill, edit the source repo/directory, then run `agentcfg preview --refresh-sources` to preview Source Refresh and `agentcfg apply --refresh-sources` to materialize it into Managed Skill Content.

## Lockfiles

Each config layer has an adjacent lockfile:

```text
${XDG_CONFIG_HOME:-~/.config}/agentcfg/lock.toml
agentcfg.lock
.agentcfg/lock.toml
```

The lockfile records exact resolved inputs:

- source id
- source type
- requested git ref, when applicable
- resolved git commit, when applicable
- Source Skill Name (original name in the Skill Source)
- Discovery Name (installed skill name exposed to clients)
- source hash
- installed hash
- alias rewrite flag
- materialized symlinks

Floating git refs are allowed in config, but lockfiles record concrete commits. Examples of floating refs include `main`, `trunk`, `develop`, and `release/2026-05`; a pinned commit SHA is not floating.

Plain `apply` uses the existing lockfile when present. `apply --refresh-sources` performs Source Refresh, refreshes Skill Source resolutions, and rewrites active lockfiles.

For path Skill Sources:

- Plain `apply` should install from the locked Managed Skill Content if available.
- If the lockfile exists but Managed Skill Content is missing, plain `apply` recreates it from the current Skill Source only when the current source materializes to the locked `source_hash`.
- If the current Skill Source is unavailable, plain `apply` must fail and ask the user to restore the source or managed state.
- If the current Skill Source is available but no longer matches the locked `source_hash`, plain `apply` must fail and tell the user to run `agentcfg apply --refresh-sources` only if they want to accept the changed source content.
- Current path Skill Source edits do not affect plain `apply` while the locked Managed Skill Content exists.
- `preview --refresh-sources` and `apply --refresh-sources` detect and use changed path Skill Source content.

For git Skill Sources, plain `apply` installs from the locked Managed Skill Content. If that copy is missing, `apply` may recreate it from the locked commit. If the locked commit cannot be fetched, `apply` must fail and ask the user to restore managed state or make the locked commit available; `apply --refresh-sources` is only appropriate when the user wants to move to a newer resolved commit.

## Manifest

The manifest records local generated state and the Discovery Requirements that keep shared Installed Artifacts alive.

Project manifest:

```text
.agentcfg/manifest.json
```

User manifest:

```text
${XDG_STATE_HOME:-~/.local/state}/agentcfg/manifest.json
```

Manifest records should include:

- kind (`skill` in V1)
- source id
- Source Skill Name
- Discovery Name
- target path
- target kind
- installed hash
- Discovery Requirements (config layer, client, install level)
- created-by marker
- source acquisition mode
- target mode

Discovery Requirements should be structured, not just a string list (serialized manifest field may remain `consumers` until schema migration):

```json
"consumers": [
  {"scope": "shared-project", "client": "codex", "install_level": "project"},
  {"scope": "user-project", "client": "codex", "install_level": "project"}
]
```

`scope` is the Persisted Scope Value for the Config Layer. This preserves shared Client Discovery Location behavior while supporting layered Discovery Requirements later.

## Managed Skill Content Cleanup

Managed Skill Content under `.agentcfg/sources/` or `${XDG_STATE_HOME:-~/.local/state}/agentcfg/sources/` is rebuildable cache derived from lockfiles, not user-authored config and not client-visible install targets.

The manifest owns Installed Artifacts at Client Discovery Locations. It does not need one record per internal Managed Skill Content tree.

Cleanup policy:

- `prune` removes stale Installed Artifacts and stale Discovery Requirements.
- `prune` may remove Managed Skill Content that is no longer used by any active lockfile or manifest-owned Installed Artifact.
- If source cleanup is risky or expensive in the first implementation slice, it may be skipped conservatively.
- `status` should be able to report unused Managed Skill Content as cache leftovers.

Never remove user-authored source directories.

## Client Discovery Registry

Client Discovery Location definitions should be built-in defaults with docs-backed provenance and confidence levels. The Client Discovery Registry is the V1 compatibility boundary between client-specific filesystem conventions and the shared planning/apply engine.

Registry entries should be keyed by `{resource_kind, client, scope}`. V1 only registers `resource_kind = "skill"`, but including the key keeps manifests, plan records, and diagnostics from baking in skill-only assumptions about target ownership.

Default policy:

- Prefer portable shared paths where officially supported.
- Use client-native paths when there is no portable path.
- Disable uncertain Client Discovery Locations by default.

V1 recommended built-ins:

| Client | Project skills | User skills | Default |
| --- | --- | --- | --- |
| Codex | `.agents/skills/{name}` | `~/.agents/skills/{name}` | enabled |
| Pi | `.agents/skills/{name}` | `~/.agents/skills/{name}` | enabled |
| OpenCode | `.agents/skills/{name}` | `~/.agents/skills/{name}` | enabled |
| Claude Code | `.claude/skills/{name}` | `~/.claude/skills/{name}` | enabled |
| Cline | `.cline/skills/{name}` | `~/.cline/skills/{name}` | enabled |
| Cursor | `.agents/skills/{name}` | `~/.agents/skills/{name}` | enabled |

Known native alternatives should be represented in the registry but not necessarily enabled by default:

- Pi: `.pi/skills/{name}`, `~/.pi/agent/skills/{name}`
- OpenCode: `.opencode/skills/{name}`, `~/.config/opencode/skills/{name}`
- Cline compatibility paths: `.clinerules/skills/{name}`, `.claude/skills/{name}`
- Cursor: `.cursor/skills/{name}`, `~/.cursor/skills/{name}`

The registry should carry confidence/provenance metadata so `doctor` can explain uncertain or experimental Client Discovery Locations. Cline's first-class skill support is experimental, so the Cline `.cline/skills/{name}` Client Discovery Location should be enabled with explicit provenance and an experimental confidence note. Do not use `.agents/skills/{name}` for Cline unless Cline documents or implements that discovery path.

Client families such as clients that share `.agents/skills/{name}` should be represented as multiple client registry entries that resolve to the same Client Discovery Location path, not as a separate family interface. Shared Client Discovery Location behavior belongs to the Discovery Requirement model.

## Shared Client Discovery Locations

When multiple clients use the same Client Discovery Location path, install one Installed Artifact and track multiple Discovery Requirements.

Example:

```text
.agents/skills/do-code-review
Discovery Requirements: codex, pi, opencode (shared-project, project level)
```

Adding a client adds Discovery Requirements. Removing a client makes those Discovery Requirements stale; `prune` removes stale Discovery Requirements and deletes the Installed Artifact only when no Discovery Requirements remain.

## Skill Aliases and Discovery Name Collisions

Discovery Names must be unique per Client Discovery Location.

If two Skill Sources select the same Source Skill Name, V1 should fail unless a Skill Alias is configured.

Example:

```toml
[skill_aliases]
"community:code-review" = "security-review"
```

Skill Alias behavior:

- Skill Alias changes the Discovery Name (runtime identity exposed to clients).
- Patch Managed Skill Content `SKILL.md` frontmatter `name`, when present, so Agent Skill Format metadata matches the Discovery Name.
- Do not mutate the upstream Skill Source.
- Emit alias rewrites in `preview`.
- Summarize alias rewrites in `apply`.

Implementation invariant:

```text
Discovery Name is the runtime identity.
```

Useful code comment:

```rust
// The Discovery Name is the identity exposed to agent clients. Some clients read
// SKILL.md frontmatter while others key off the directory name, so Skill Aliases must
// rewrite Managed Skill Content into one consistent identity. agentcfg must not
// mutate the Skill Source tree.
```

Preserve Source Skill Names in lockfile and manifest for debugging.

## Hashing

Use SHA-256 in V1.

Hash strings should include an algorithm prefix:

```text
sha256:<hex>
```

Hash the deterministic materialized skill directory tree, not only `SKILL.md`.

Materialization order:

```text
source tree -> safe materialized tree -> alias rewrite -> installed tree
```

Hashes:

- `source_hash`: materialized tree before alias rewrite.
- `installed_hash`: materialized tree after alias rewrite.

Deterministic tree hash contract:

1. Walk the skill directory recursively.
2. Materialize safe internal symlinks.
3. Reject external symlinks.
4. Reject special files.
5. Keep regular files only.
6. Normalize relative paths to POSIX-style `/`.
7. Sort entries lexicographically by normalized path.
8. For each entry, feed length-prefixed path bytes and length-prefixed content bytes.
9. Return SHA-256 hex with `sha256:` prefix.

Document the hash contract in a dedicated doc before relying on it for compatibility.

## Symlink and Filesystem Safety

No hard skill size limits in V1.

Symlink rules for all source types:

- Internal symlink that resolves inside the same skill directory: materialize as regular file/directory content in the managed copy.
- External symlink that resolves outside the skill directory: reject.
- Broken symlink: reject.
- Special files: reject.

Special files include:

- sockets
- named pipes
- block devices
- character devices

Reason: these are not portable skill content and may hang, expose system resources, or have no deterministic content.

## Init, Status, and Doctor Details

`init` should be conservative:

- create config for the selected scope
- create `.agentcfg/` when needed
- detect and report existing unmanaged Installed Artifacts
- not adopt, overwrite, or delete existing artifacts
- not write Client Discovery Location directories; `apply` does that

Existing Installed Artifacts at Client Discovery Locations during init are unmanaged:

```text
Found existing unmanaged artifacts:
  .agents/skills/do-code-review

agentcfg will not modify or remove them.
```

Adoption/import is out of V1 unless a concrete migration need forces it.

`status` checks:

- installed managed artifacts by client
- broken symlinks
- unexpected symlink targets
- missing Managed Skill Content
- stale managed artifacts
- unmanaged artifacts in configured Client Discovery Locations, reported as informational unless they conflict with desired managed Installed Artifacts
- config/lock mismatch
- manifest readability

`doctor` checks:

- git availability
- repo root detection
- known/supported clients
- path writability
- config schema validity
- optional network/source checks
- Client Discovery Location confidence warnings
- unmanaged artifacts only when they affect environment health, such as blocking a planned Client Discovery Location path

`doctor` may be slower and more explanatory. `status` should be fast, local, and scriptable.

## Implementation Boundary

Keep implementation design skill-first. The planner/apply boundary is the important one:

```text
resolve resource-specific desired state -> desired Installed Artifacts -> build plan -> render plan OR apply plan
```

`preview` and `apply` should share the same planner.

V1 has one resource-specific resolver: skills. Skill resolution owns config parsing, source discovery, group expansion, aliases, materialization, and skill hashes. After that, the shared planner/apply/status/prune machinery should operate on structured desired Installed Artifacts:

- `kind = "skill"`
- Client Discovery Location path and target mode (internal manifest field)
- Managed Skill Content path
- Discovery Name and installed hash
- source/layer provenance
- Discovery Requirements keyed by Config Layer, Client, and Install Level

This keeps the current implementation skill-first while avoiding skill-specific duplication in discovery planning, manifest safety, and client diagnostics.

## Cargo Workspace Boundary

V1 should use a single Cargo workspace with separate CLI and core library crates:

```text
crates/agentcfg-cli/
crates/agentcfg-core/
```

The `agentcfg-cli` crate owns the command-line interface:

- argument parsing
- terminal rendering
- exit codes
- command-specific user interaction

The `agentcfg-core` crate owns the reusable skill-management engine:

- config discovery, parsing, and validation
- source discovery and resolution
- safe materialization and hashing
- lockfile and manifest models
- desired-state planning
- apply, prune, status, and doctor operations
- built-in Client Discovery Registry
- filesystem safety invariants

The published binary name remains:

```text
agentcfg
```

The core crate is the boundary for future non-CLI interfaces such as a TUI, desktop UI, editor integration, daemon, or tests that need structured plan/status results. Those interfaces should call the same planner and operation APIs as the CLI instead of shelling out to the `agentcfg` binary or duplicating planner logic.

Do not make the core crate a broad generic platform in V1. Its public surface should remain skill-first and should expose structured domain results rather than terminal-formatted text. A separately branded `agentcfg-sdk` crate or stabilized external API can be added later if real downstream integrators need stronger compatibility guarantees.

## Future Resource Types

V1 should be implemented as a skill-first resolver plus shared discovery planning and application. It is acceptable for manifest or plan records to include `kind = "skill"` and for the Client Discovery Registry to key entries by resource kind, but do not build generic resource manager traits, factories, or interfaces before a second resource kind exists.

Future versions may add resource kinds such as MCP servers, hooks, rules, commands, and workflows. MCP should be a separate resource kind later, not a projection of a skill.

## Post-V1 Design Holding Area

These are intentionally deferred decisions. They are not V1 commitments, but V1 should avoid choices that make them hard.

### Resource-specific CLI selectors

The default command meaning should be aggregate desired-state apply:

```text
agentcfg apply = apply all configured resource types for the selected scope
```

In V1, skills are the only configured resource type, so `agentcfg apply` only installs skills as a consequence of the V1 resource set. Do not define the command's meaning as "apply skills"; that would make adding MCP or other resources later a semantic change.

Possible future narrowing commands:

```text
agentcfg preview skills
agentcfg apply skills
agentcfg status skills
agentcfg prune skills

agentcfg preview mcp
agentcfg apply mcp
agentcfg status mcp
```

These commands should narrow the aggregate workflow to one resource kind. They should not be required for normal setup.

### Additional resource kinds

Future resource kinds may include MCP servers, hooks, rules, commands, and workflows. Each resource kind should own its own config parsing, source resolution, conflict rules, safety rules, and apply behavior. Shared core code may coordinate structured plan/status/apply results, but it should not force every resource into the skill Installed Artifact model.

### MCP design questions

Deferred MCP work should answer:

- how server identity and conflicts are represented
- how environment variables and secrets are referenced without leaking into shared config or lockfiles
- whether MCP sync writes project config, user config, client-native config, or some combination
- how dry-run, rollback, and unmanaged-edit safety work for JSON or settings-file edits
- whether MCP sources need their own source acquisition and lock model or only desired server records
