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

## Config Layers and Install Scopes

V1 exposes three config layers:

| Human-facing name | Persisted `scope` value | Core name | Meaning |
| --- | --- | --- | --- |
| user config | `user` | `ConfigLayer::User` | Current user's home/config layer. |
| shared project config | `sharedProject` | `ConfigLayer::SharedProject` | Shared repo-level config. |
| user project config | `userProject` | `ConfigLayer::UserProject` | Current user's config for one repo. |

Active layers by command:

| Command | Active config layers | Install scope |
| --- | --- | --- |
| `agentcfg plan` | shared project config, then user project config | project |
| `agentcfg plan --upgrade` | shared project config, then user project config | project |
| `agentcfg sync` | shared project config, then user project config | project |
| `agentcfg sync --upgrade` | shared project config, then user project config | project |
| `agentcfg prune` | manifest consumers for project targets | project |
| `agentcfg status` | shared project config, user project config, project manifest | project |
| `agentcfg doctor` | all discoverable config and environment state | diagnostic only |
| `agentcfg plan --user` | user config only | user |
| `agentcfg plan --user --upgrade` | user config only | user |
| `agentcfg sync --user` | user config only | user |
| `agentcfg sync --user --upgrade` | user config only | user |
| `agentcfg prune --user` | stale consumers and artifacts in user targets | user |
| `agentcfg status --user` | user config and user manifest | user |

Layer order:

```text
shared project config -> user project config
```

User project config is additive by default. It may add sources, selected skills, aliases, and clients for the current user in the current repo. It must not silently replace or weaken shared project config. Explicit override semantics are out of V1.

Source ids are namespaced by layer internally:

```text
{scope}:{source_id}
```

This allows shared project config and user project config to both use a source id such as `local` without collision. User-facing diagnostics should include both the human source id and the layer when ambiguity matters.

Installed names are not namespaced. After alias resolution, installed skill names must be unique per target path. If two active layers resolve to the same installed name and same target path:

- If they refer to the same locked source skill and same installed hash, merge consumers.
- If they differ, fail with a collision error and require an alias.

Aliases are applied before collision detection.

Client selection is additive across active layers. If shared project config selects `codex` and user project config selects `opencode`, the desired project install includes both clients. If CLI `--client` is omitted, install-scoped commands use the full configured client set for the active layers. CLI `--client` may be repeated to narrow that configured client set, but must not add clients outside the configured selection in V1.

Consumers are tracked by `{scope, client}`. Removing a skill or client from one layer makes that consumer stale. `prune` removes stale consumers and deletes the target artifact only when no consumers remain.

In code, use `ConfigLayer` for the config file/layer being initialized or loaded, `InstallScope` for project-vs-user target installation, and reserve `target` for concrete client target artifacts such as target paths and target modes.

Use `SourceResolutionPolicy::{UseLocked, RefreshSources}` for the core source-resolution choice so the workflow API does not leak the CLI flag name `--upgrade`.

## Config Schema

V1 config is skill-specific. Do not expose or implement a generic resource schema in V1.

Example:

```toml
scope = "userProject"

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
- Skill selection lives only under `[[skill_sources]]`.
- `include` is an optional list of skill names from that source.
- `groups` is an optional list of source-local group names from that source's `skills.toml`.
- If neither `include` nor `groups` is set, select all discovered skills from that source.
- `exclude` is out of V1.
- Missing `[skills].clients` is a validation error.
- `[skills].clients` may be either an explicit non-empty list of client ids or the string `"all"`.
- `clients = "all"` means every `agentcfg`-supported client target that is enabled for the selected install scope in the current `agentcfg` version. It does not mean every agent application installed on the machine. Because this can expand when `agentcfg` adds support for new clients, `plan` must show the resolved client set before `sync` writes any new target.
- `[skills]` owns install-wide skill behavior such as target clients. It does not select skill names in V1.
- CLI `--client` may narrow configured clients, but should not expand beyond configured clients in V1. If omitted, all configured clients remain selected. With `clients = "all"`, `--client` may narrow to any supported client.
- Aliases live under `[skill_aliases]`, are local to the config layer that declares them, and use qualified `source_id:skill_name` keys.
- Aliases are applied after source-local group expansion and before installed-name collision detection.

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

- Git sources are copied into the active scope's managed source directory.
- Path sources are also copied by default.
- Direct source symlink mode is deferred from V1. This would mean pointing a client target directly at the original source directory instead of a managed source tree.

Source manifests:

- `skills.toml` inside a source is optional.
- Skills can be inferred from directories containing `SKILL.md`.
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

- `copy`: default for path and git sources. The selected skill is materialized into the active scope's managed source directory.

Managed materialized tree:

```text
project sync: .agentcfg/sources/<layer>/<source-id>/<resolved-id>/<installed-name>/
user sync:    ${XDG_STATE_HOME:-~/.local/state}/agentcfg/sources/<layer>/<source-id>/<resolved-id>/<installed-name>/
```

The exact path format can change during implementation, but it must be stable enough for manifests and diagnostics. The managed tree is the canonical installed content for copied sources.

Why managed source trees are required:

- Plain `sync` can reinstall the locked skill version without rereading a mutable path source or floating git ref.
- `sync --upgrade` materializes a new managed tree, updates the lockfile, and retargets client symlinks to the new tree.
- Old managed trees can remain as cache until no lockfile or manifest target uses them, then `prune` may remove them.

Installed target mode:

- V1 default is `symlink` from the client target path to the managed materialized tree.
- V1 does not need user-configurable target mode.
- A future version may add copied target directories if a client proves incompatible with symlinked skills.

For copied sources:

```text
source -> managed materialized tree -> client target symlink
```

Client target symlinks must point to the managed materialized tree for the same sync scope:

- project targets point into project generated state under `.agentcfg/sources/`
- user targets point into user generated state under `${XDG_STATE_HOME:-~/.local/state}/agentcfg/sources/`

Expected symlink target validation:

- Manifest records the expected target for each symlinked client artifact.
- `status` reports missing links, broken targets, and unexpected symlink targets.
- `prune` refuses to remove a symlink whose current target does not match the manifest.
- `sync` may update a manifest-owned symlink when the previous target matches the manifest and the desired target changed.

Sync direction is one-way:

```text
source -> managed source tree -> client target
```

`agentcfg` never writes changes back to a skill source. To improve a skill, edit the source repo/directory, then run `agentcfg plan --upgrade` to preview the import and `agentcfg sync --upgrade` to materialize it into managed state.

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
- original skill name
- installed skill name
- source hash
- installed hash
- alias rewrite flag
- materialized symlinks

Floating git refs are allowed in config, but lockfiles record concrete commits. Examples of floating refs include `main`, `trunk`, `develop`, and `release/2026-05`; a pinned commit SHA is not floating.

Plain `sync` uses the existing lockfile when present. `sync --upgrade` refreshes source resolutions and rewrites active lockfiles.

For path sources:

- Plain `sync` should install from the locked managed copy if available.
- If the lockfile exists but the managed source tree is missing, plain `sync` recreates it from the current source only when the current source materializes to the locked `source_hash`.
- If the current source is unavailable, plain `sync` must fail and ask the user to restore the source or managed state.
- If the current source is available but no longer matches the locked `source_hash`, plain `sync` must fail and tell the user to run `agentcfg sync --upgrade` only if they want to accept the changed source content.
- Current path source edits do not affect plain `sync` while the locked managed source tree exists.
- `plan --upgrade` and `sync --upgrade` detect and use changed path source content.

For git sources, plain `sync` installs from the locked managed copy. If that copy is missing, `sync` may recreate it from the locked commit. If the locked commit cannot be fetched, `sync` must fail and ask the user to restore managed state or make the locked commit available; `sync --upgrade` is only appropriate when the user wants to move to a newer resolved commit.

## Manifest

The manifest records local generated state and the consumers that keep shared target artifacts alive.

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
- original name
- installed name
- target path
- target kind
- installed hash
- consuming layer/client pairs
- created-by marker
- source acquisition mode
- target mode

Consumers should be structured, not just a string list:

```json
"consumers": [
  {"scope": "project", "client": "codex"},
  {"scope": "userProject", "client": "codex"}
]
```

This preserves shared-target behavior while supporting layered consumers later.

## Managed Source State Cleanup

Managed source trees under `.agentcfg/sources/` or `${XDG_STATE_HOME:-~/.local/state}/agentcfg/sources/` are rebuildable cache derived from lockfiles, not user-authored config and not client-visible install targets.

The manifest owns client target artifacts. It does not need one record per internal managed source tree.

Cleanup policy:

- `prune` removes stale target artifacts and stale consumers.
- `prune` may remove managed source trees that are no longer used by any active lockfile or manifest-owned target.
- If source cleanup is risky or expensive in the first implementation slice, it may be skipped conservatively.
- `status` should be able to report unused managed source trees as cache leftovers.

Never remove user-authored source directories.

## Client Target Registry

Client target definitions should be built-in defaults with docs-backed provenance and confidence levels. The registry is the V1 compatibility boundary between client-specific filesystem conventions and the shared planning/apply engine.

Registry entries should be keyed by `{resource_kind, client, scope}`. V1 only registers `resource_kind = "skill"`, but including the key keeps manifests, plan records, and diagnostics from baking in skill-only assumptions about target ownership.

Default policy:

- Prefer portable shared paths where officially supported.
- Use client-native paths when there is no portable path.
- Disable uncertain targets by default.

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

The registry should carry confidence/provenance metadata so `doctor` can explain uncertain or experimental targets. Cline's first-class skill support is experimental, so the Cline `.cline/skills/{name}` target should be enabled with explicit provenance and an experimental confidence note. Do not use `.agents/skills/{name}` for Cline unless Cline documents or implements that discovery path.

Client families such as clients that share `.agents/skills/{name}` should be represented as multiple client registry entries that resolve to the same target path, not as a separate family interface. Shared target behavior belongs to the consumer model.

## Shared Target Consumers

When multiple clients use the same target path, install one artifact and track multiple consumers.

Example:

```text
.agents/skills/do-code-review
consumers: codex, pi, opencode
```

Adding a client adds consumers. Removing a client makes those consumers stale; `prune` removes stale consumers and deletes the artifact only when no consumers remain.

## Aliases and Collisions

Installed skill names must be unique per target path.

If two sources select the same skill name, V1 should fail unless an alias is configured.

Example:

```toml
[skill_aliases]
"community:code-review" = "security-review"
```

Alias behavior:

- Alias changes the installed runtime name.
- Patch the managed copy's `SKILL.md` frontmatter `name`, when present.
- Do not mutate the upstream source.
- Emit alias rewrites in `plan`.
- Summarize alias rewrites in `sync`.

Implementation invariant:

```text
Installed name is the runtime identity.
```

Useful code comment:

```rust
// The installed name is the identity exposed to agent clients. Some clients read
// SKILL.md frontmatter while others key off the directory name, so aliases must
// rewrite the managed copy into one consistent identity. agentcfg must not
// mutate the source tree.
```

Preserve original names in lockfile and manifest for debugging.

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
- detect and report existing unmanaged client artifacts
- not adopt, overwrite, or delete existing artifacts
- not write target client directories; `sync` does that

Existing client target artifacts during init are unmanaged:

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
- missing managed sources
- stale managed artifacts
- unmanaged artifacts in configured target directories, reported as informational unless they conflict with desired managed targets
- config/lock mismatch
- manifest readability

`doctor` checks:

- git availability
- repo root detection
- known/supported clients
- path writability
- config schema validity
- optional network/source checks
- target confidence warnings
- unmanaged artifacts only when they affect environment health, such as blocking a planned target path

`doctor` may be slower and more explanatory. `status` should be fast, local, and scriptable.

## Implementation Boundary

Keep implementation design skill-first. The planner/apply boundary is the important one:

```text
resolve resource-specific desired state -> desired target artifacts -> build plan -> render plan OR apply plan
```

`plan` and `sync` should share the same planner.

V1 has one resource-specific resolver: skills. Skill resolution owns config parsing, source discovery, group expansion, aliases, materialization, and skill hashes. After that, the shared planner/apply/status/prune machinery should operate on structured desired target artifacts:

- `kind = "skill"`
- target path and target mode
- managed source path
- installed name and installed hash
- source/layer provenance
- consuming `{scope, client}` pairs

This keeps the current implementation skill-first while avoiding skill-specific duplication in target planning, manifest safety, and client diagnostics.

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
- sync, prune, status, and doctor operations
- built-in client target registry
- filesystem safety invariants

The published binary name remains:

```text
agentcfg
```

The core crate is the boundary for future non-CLI interfaces such as a TUI, desktop UI, editor integration, daemon, or tests that need structured plan/status results. Those interfaces should call the same planner and operation APIs as the CLI instead of shelling out to the `agentcfg` binary or duplicating planner logic.

Do not make the core crate a broad generic platform in V1. Its public surface should remain skill-first and should expose structured domain results rather than terminal-formatted text. A separately branded `agentcfg-sdk` crate or stabilized external API can be added later if real downstream consumers need stronger compatibility guarantees.

## Future Resource Types

V1 should be implemented as a skill-first resolver plus shared target planning and application. It is acceptable for manifest or plan records to include `kind = "skill"` and for the client registry to key entries by resource kind, but do not build generic resource manager traits, factories, or interfaces before a second resource kind exists.

Future versions may add resource kinds such as MCP servers, hooks, rules, commands, and workflows. MCP should be a separate resource kind later, not a projection of a skill.

## Post-V1 Design Holding Area

These are intentionally deferred decisions. They are not V1 commitments, but V1 should avoid choices that make them hard.

### Resource-specific CLI selectors

The default command meaning should be aggregate desired-state sync:

```text
agentcfg sync = sync all configured resource types for the selected scope
```

In V1, skills are the only configured resource type, so `agentcfg sync` only installs skills as a consequence of the V1 resource set. Do not define the command's meaning as "sync skills"; that would make adding MCP or other resources later a semantic change.

Possible future narrowing commands:

```text
agentcfg plan skills
agentcfg sync skills
agentcfg status skills
agentcfg prune skills

agentcfg plan mcp
agentcfg sync mcp
agentcfg status mcp
```

These commands should narrow the aggregate workflow to one resource kind. They should not be required for normal setup.

### Additional resource kinds

Future resource kinds may include MCP servers, hooks, rules, commands, and workflows. Each resource kind should own its own config parsing, source resolution, conflict rules, safety rules, and apply behavior. Shared core code may coordinate structured plan/status/apply results, but it should not force every resource into the skill target artifact model.

### MCP design questions

Deferred MCP work should answer:

- how server identity and conflicts are represented
- how environment variables and secrets are referenced without leaking into shared config or lockfiles
- whether MCP sync writes project config, user config, client-native config, or some combination
- how dry-run, rollback, and unmanaged-edit safety work for JSON or settings-file edits
- whether MCP sources need their own source acquisition and lock model or only desired server records
