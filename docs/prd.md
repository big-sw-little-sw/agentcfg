# agentcfg PRD

## Purpose

`agentcfg` is a CLI for managing agent configuration as repeatable desired state, starting with skills.

V1 should be source-agnostic, explicit about lifecycle state, and compatible with skills in **Agent Skill Format** (`SKILL.md` directories). It should manage skills well without becoming a full cross-agent configuration platform.

Detailed persisted contracts and safety rules live in [design-v1.md](design-v1.md).

## Goals

- Separate the skill manager from skill repositories.
- Consume skills from filesystem path and git **Skill Sources**.
- Keep skills in **Agent Skill Format** (`SKILL.md` directories).
- Support repeatable `preview` and `apply` behavior.
- Support Shared Project Config, User Project Config, and User Config layers.
- Preserve manifest-owned cleanup safety.
- Prefer portable skill paths where agent clients officially support them.
- Avoid imposing committed agent config on teams unless explicitly requested.

## Non-Goals for V1

- No command/workflow/rule projections.
- No MCP management.
- No registry publishing.
- No desktop UI.
- No arbitrary org/team layer discovery in the public CLI.
- No hard size limits on skills.

## Terms

- **Client**: an agent application or CLI that discovers skills from filesystem paths, such as Codex, Pi, OpenCode, Claude Code, Cline, or Cursor.
- **Client Discovery Location**: a client-specific filesystem location where `agentcfg` installs a skill entrypoint, such as `.agents/skills/{name}`.
- **Skill Source**: a filesystem path or git location containing skill directories in Agent Skill Format.
- **Managed Skill Content**: generated copy of resolved skill content under `agentcfg` state. Client Discovery Locations point to this content so normal apply can install the locked version without depending on a mutable source path or moving git branch.
- **Skill Selection**: per-source `include` (selects **Included Skills**) and `groups` (selects **Skill Groups**).
- **Skill Alias**: maps a Source Skill Name to a **Discovery Name**; may require Managed Skill Content frontmatter preparation.
- **Discovery Name Collision**: two active layers resolve the same Discovery Name at the same Client Discovery Location with different content.
- **Config layer**: one Config Layer participating in preview or apply, such as Shared Project Config or User Project Config.
- **Install level**: whether a command installs or inspects at Project Level or User Level.
- **Discovery Requirement**: a manifest record keyed by Config Layer, Client, and Install Level that says which layer/client requires an Installed Artifact. A shared Client Discovery Location can be pruned only when it has no remaining Discovery Requirements.
- **Installed Artifact**: a manifest-owned skill entry at a Client Discovery Location (symlink or copy to managed content).
- **Client selector**: an optional CLI filter that narrows a command at a given Install Level to one or more configured clients.

## Config Types

V1 has three user-facing config types:

- **Shared project config**: `agentcfg.toml` at the repo root. This is committed when a repo intentionally wants common agent skills for everyone working in that project. Its lockfile is `agentcfg.lock`.
- **User project config**: `.agentcfg/config.toml` inside a repo. This is for one user's additions in that specific project and should stay uncommitted with the rest of `.agentcfg/`. Its lockfile is `.agentcfg/lock.toml`.
- **User config**: `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`. This is for one user's skills across projects and applies only to user-level Client Discovery Locations. Its lockfile is `${XDG_CONFIG_HOME:-~/.config}/agentcfg/lock.toml`.

Project apply reads shared project config plus user project config. User config is applied separately with `agentcfg apply --user`; it is not merged into project apply.

Rationale: agent clients already define how user-level and project-level skills are combined at runtime. `agentcfg` should install user config to user-level Client Discovery Locations and project config to project-level Client Discovery Locations, then let each client apply its own merge and precedence rules. This avoids duplicating user skills into every project and avoids second-guessing client-specific behavior.

## Commands

V1 commands:

```bash
agentcfg init
agentcfg init --project
agentcfg init --user

agentcfg preview
agentcfg preview --refresh-sources
agentcfg preview --user
agentcfg preview --user --refresh-sources
agentcfg preview --client <client>

agentcfg apply
agentcfg apply --refresh-sources
agentcfg apply --user
agentcfg apply --user --refresh-sources
agentcfg apply --client <client>

agentcfg prune
agentcfg prune --user
agentcfg prune --client <client>
agentcfg status
agentcfg status --user
agentcfg status --client <client>
agentcfg doctor
```

Command semantics:

- `init`: create config for the selected config layer. Default creates user project config at `.agentcfg/config.toml`.
- `init --project`: create shared project config at `agentcfg.toml`.
- `init --user`: create user config at `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`.
- `preview`: strict read-only preview. No persistent writes to config, lockfiles, manifests, sources, caches, or Client Discovery Locations.
- `preview --refresh-sources`: read-only preview after **Source Refresh** (refreshing Skill Source resolutions in memory). For git Skill Sources, this means checking whether floating refs moved. For path Skill Sources, this means checking whether source content changed.
- `apply`: create missing lockfiles if needed, then install the locked resolved state.
- `apply --refresh-sources`: perform Source Refresh, update active lockfiles, materialize refreshed Managed Skill Content, then install.
- `prune`: remove stale managed Installed Artifacts and stale Discovery Requirements. It applies by default because `preview` is the read-only workflow command.
- `status`: inspect current managed install state.
- `doctor`: diagnose environment, client support, config validity, path writability, and optional network/source issues.
- `--user` on `init`: create User Config. `--user` on `preview`, `apply`, `prune`, and `status`: run at User Level (user Install Level) using User Config and user-level Client Discovery Locations. Not used for `doctor`, which is diagnostic.
- `--client <client>`: narrow `preview`, `apply`, `prune`, or `status` to the named client. It may be repeated. If omitted, the command applies to all clients selected by the active config layers. `--client` must not add a client outside the configured selection in V1. If config uses `clients = "all"`, any supported client may be selected.

## User Workflows

User project setup:

```bash
agentcfg init
agentcfg preview
agentcfg apply
```

Shared project setup:

```bash
agentcfg init --project
agentcfg preview
agentcfg apply
```

User-level setup:

```bash
agentcfg init --user
agentcfg preview --user
agentcfg apply --user
```

Source Refresh for selected Skill Sources:

```bash
agentcfg preview --refresh-sources
agentcfg apply --refresh-sources
```

Remove stale managed artifacts:

```bash
agentcfg preview
agentcfg prune
```

## Preview, Apply, and Prune

`preview` should show:

- lockfile changes that would be created or updated
- source resolutions
- skills to create
- skills to update
- Discovery Requirement additions
- stale Discovery Requirement removals
- stale Installed Artifact removals
- alias rewrites
- warnings for uncertain Client Discovery Locations

`apply` applies:

- missing lockfile creation
- Client Discovery Location creates
- Client Discovery Location updates
- Discovery Requirement additions

`apply` does not remove stale artifacts. If stale artifacts remain, it warns:

```text
Stale managed artifacts remain: N
These may still be discovered by agent clients until pruned.
Run: agentcfg prune
```

`prune` applies:

- stale Discovery Requirement removals
- stale managed Installed Artifact removals

Cleanup safety invariants:

- Remove only manifest-owned artifacts.
- Refuse unexpected symlink targets.
- Never delete unmanaged real files.
- Delete directories only if empty and manifest-owned.

## Supported Clients

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

Known native alternatives are design details covered in [design-v1.md](design-v1.md).

## Status and Doctor

`status` answers:

```text
Is the current managed install state consistent?
```

It should report installed managed Installed Artifacts, broken symlinks, unexpected symlink targets, missing Managed Skill Content, stale managed Installed Artifacts, config/lock mismatch, manifest readability, and informational unmanaged artifacts in configured Client Discovery Locations.

`doctor` answers:

```text
Is my environment/config/tooling capable of working?
```

It should check git availability, repo root detection, supported clients, path writability, config schema validity, optional network/source checks, Client Discovery Location confidence warnings, and unmanaged artifacts only when they block planned Client Discovery Location paths.

## MVP Acceptance Criteria

- Automated tests pass.
- `agentcfg init` creates `.agentcfg/config.toml`.
- `agentcfg init --project` creates `agentcfg.toml`.
- `agentcfg init --user` creates `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`.
- `agentcfg preview` is read-only.
- `agentcfg apply` resolves a path Skill Source, writes lockfile, materializes Managed Skill Content, and installs skill symlink.
- `agentcfg apply --refresh-sources` performs Source Refresh for path Skill Source content and lockfile.
- `agentcfg prune` removes only manifest-owned stale artifacts.
- Alias collision handling is tested.
- Internal symlink materialization and external symlink rejection are tested.
- Shared `.agents/skills` Discovery Requirements across Codex/Pi/OpenCode/Cursor are tested.

## Open Product Questions

- Whether git sources are in the first implementation slice or come after path sources - Answered: YES
- How much source provenance to expose for Client Discovery Registry decisions.
- Whether both `skills/<name>/SKILL.md` and root-level `<name>/SKILL.md` source layouts should be accepted in V1.
