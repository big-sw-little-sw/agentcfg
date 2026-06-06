# agentcfg PRD

## Purpose

`agentcfg` is a CLI for managing Agent Configuration as repeatable pinned configuration and managed installation, starting with skills.

V1 should support multiple Skill Source kinds, be explicit about configuration pinning and observed installation lifecycle, and stay compatible with skills in **Agent Skill Format** (`SKILL.md` directories). It should manage skills well without becoming a full platform for every agent-facing configuration type. Future subagent support should be a separate Configured Item path, not an expansion of Skill Configuration V1.

Detailed persisted contracts and safety rules live in [design-v1.md](design-v1.md).

## Goals

- Separate Skill Configuration handling from Skill Sources.
- Consume skills from filesystem path and git **Skill Sources**.
- Keep skills in **Agent Skill Format** (`SKILL.md` directories).
- Support repeatable `preview` and `apply` behavior.
- Support Shared Project Config, User Project Config, and User Config layers.
- Preserve manifest-owned cleanup safety.
- Prefer portable Client Discovery Locations where Clients officially support them.
- Avoid imposing committed Agent Configuration on teams unless explicitly requested.

## Non-Goals for V1

- No command/workflow/rule projections.
- No MCP management.
- No external Skill catalog publishing.
- No desktop UI.
- No arbitrary org/team layer discovery in the public CLI.
- No hard size limits on skills.
- No subagent management in V1.

## Terms

- **Configured Item**: one kind of agent-facing thing managed by `agentcfg`. V1 has one Configured Item kind: **Skill**.
- **ConfigDoc**: the parsed persisted configuration schema for one Config Layer.
- **ConfigRequest**: the normalized command-time request built from active ConfigDocs and command options before Skill Source refs, selections, aliases, and content identities are fixed.
- **PinnedConfig**: a ConfigRequest after Skill Source refs, Skill Selections, Skill Aliases, Discovery Names, and content identities are fixed.
- **LockfilePinnedConfig**: a PinnedConfig loaded from current lockfiles.
- **PlannedPinnedConfig**: a PinnedConfig proposed by preview or apply resolution.
- **Lockfile**: a user-visible file beside each Config Layer config that records **PinnedConfig** for Configured Items that need repeatable Skill Source resolution.
- **ObservedInstallation**: observed install reality from the filesystem, Manifest, Managed State, and Client Discovery Locations. It may drift from **LockfilePinnedConfig**.
- **Manifest**: an `agentcfg`-owned record under **Managed State** that records **Installed Artifacts** and the **Discovery Requirements** that keep them present.
- **Managed State**: `agentcfg`-owned state used to apply, inspect, and prune configuration safely, including the Manifest and **Managed Skill Content**.
- **Client**: an agent application or CLI that discovers Configured Items from Client Discovery Locations, such as Codex, Pi, OpenCode, Claude Code, Cline, or Cursor.
- **Client Discovery Location**: a client-specific filesystem location that a Client scans to discover managed configuration, such as `.agents/skills/{name}`.
- **Skill Source**: a filesystem path or git location containing skill directories in Agent Skill Format.
- **Managed Skill Content**: `agentcfg`-owned skill files prepared from PinnedConfig under Managed State. Client Discovery Locations point to this content so normal apply can install pinned content without depending on a mutable Skill Source path or moving git branch.
- **Skill Selection**: per-Skill Source `include` (selects **Included Skills**) and `groups` (selects **Skill Groups**).
- **Skill Alias**: maps a Source Skill Name to a **Discovery Name**; may require Managed Skill Content frontmatter preparation.
- **Discovery Name Collision**: two Active Config Layers resolve the same Discovery Name at the same Client Discovery Location with different content.
- **Config Layer**: one Config Layer participating in preview or apply, such as Shared Project Config or User Project Config.
- **Install Level**: whether a command installs or reports at Project Level or User Level.
- **Discovery Requirement**: a manifest record keyed by Config Layer, Client, and Install Level that says which layer/client requires an Installed Artifact. A shared Client Discovery Location can be pruned only when it has no remaining Discovery Requirements.
- **Installed Artifact**: a manifest-owned skill entry at a Client Discovery Location (symlink or copy to **Managed Skill Content**).
- **Unmanaged Artifact**: a filesystem entry at a Client Discovery Location that is not recorded in the Manifest as owned by `agentcfg`.
- **Stale Discovery Requirement**: a Discovery Requirement recorded in the Manifest that is no longer present in the PinnedConfig being reconciled.
- **Unsatisfied Discovery Requirement**: a Discovery Requirement in the PinnedConfig being reconciled that does not have a valid Installed Artifact in ObservedInstallation.
- **Stale Installed Artifact**: an Installed Artifact recorded in the Manifest that has no remaining Discovery Requirements.
- **Unexpected Symlink Target**: a symlink destination that differs from the destination recorded in the Manifest for an Installed Artifact.
- **Broken Symlink**: an Installed Artifact symlink whose destination does not exist.
- **Client selector**: an optional CLI filter that narrows a command at a given Install Level to one or more configured clients.
- **ApplyPlan**: the concrete install mutation plan derived by comparing a PlannedPinnedConfig with ObservedInstallation.
- **ApplyResult**: the outcome of executing an ApplyPlan, including completed changes, blockers, failures, and recovery diagnostics.

## Config Layers

V1 has three user-facing Config Layers:

- **Shared Project Config**: `agentcfg.toml` at the Project Root. This is committed when a Project intentionally wants common agent skills for everyone working in that Project. Its lockfile is `agentcfg.lock`.
- **User Project Config**: `.agentcfg/config.toml` inside a Project. This is for one User's additions in that specific Project and should stay uncommitted with the rest of `.agentcfg/`. Its lockfile is `.agentcfg/lock.toml`.
- **User Config**: `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`. This is for one User's skills across Projects and applies only to user-level Client Discovery Locations. Its lockfile is `${XDG_CONFIG_HOME:-~/.config}/agentcfg/lock.toml`.

Project Level apply reads Shared Project Config plus User Project Config. User Config is applied separately with `agentcfg apply --user`; it is not merged into Project Level apply.

Rationale: Clients already define how user-level and project-level skills are combined at runtime. `agentcfg` should install User Config to user-level Client Discovery Locations and Project Level configuration to project-level Client Discovery Locations, then let each client apply its own merge and precedence rules. This avoids duplicating user skills into every project and avoids second-guessing client-specific behavior.

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

- `init`: create config for the selected Config Layer. Default creates User Project Config at `.agentcfg/config.toml`.
- `init --project`: create Shared Project Config at `agentcfg.toml`.
- `init --user`: create User Config at `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`.
- `preview`: strict read-only preview of the active ConfigRequest, PlannedPinnedConfig, and install changes. No persistent writes to config, lockfiles, the Manifest, Managed State (including Managed Skill Content), Skill Sources, or Client Discovery Locations.
- `preview --refresh-sources`: read-only preview after **Source Refresh** (refreshing Skill Source resolutions in memory). For git Skill Sources, this means checking whether floating refs moved. For path Skill Sources, this means checking whether Skill Source content changed.
- `apply`: create missing lockfiles if needed, then install **PlannedPinnedConfig** into Managed State and Client Discovery Locations.
- `apply --refresh-sources`: perform Source Refresh, update active lockfiles, materialize refreshed Managed Skill Content, then install.
- `prune`: remove **Stale Installed Artifacts** and **Stale Discovery Requirements** from Managed State. It applies by default because `preview` is the read-only workflow command.
- `status`: report managed install-state consistency for the active Install Level.
- `doctor`: check environment and configuration readiness (client support, config validity, path writability, and optional network/Skill Source issues). It does not report managed install-state consistency; use `status` for that.
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

Remove **Stale Installed Artifacts** and **Stale Discovery Requirements**:

```bash
agentcfg preview
agentcfg prune
```

## Preview, Apply, and Prune

`preview` should show:

- lockfile changes that would be created or updated
- Skill Source resolutions used for PlannedPinnedConfig
- **Installed Artifact** creates in the ApplyPlan
- **Installed Artifact** updates in the ApplyPlan
- Discovery Requirement additions
- stale Discovery Requirement removals
- stale Installed Artifact removals
- Discovery Name preparation
- warnings for uncertain Client Discovery Locations

`apply` applies:

- missing lockfile creation
- Managed Skill Content materialization
- **Installed Artifact** creates at Client Discovery Locations
- **Installed Artifact** updates at Client Discovery Locations
- Discovery Requirement additions

`apply` does not remove **Stale Installed Artifacts** or **Stale Discovery Requirements**. If stale Managed State remains, it warns:

```text
Stale Installed Artifacts or Stale Discovery Requirements remain: N
These may still be discovered by Clients until pruned.
Run: agentcfg prune
```

`prune` applies:

- **Stale Discovery Requirement** removals
- **Stale Installed Artifact** removals

Cleanup safety invariants:

- Remove only manifest-owned artifacts.
- Refuse **Unexpected Symlink Target** destinations.
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
Is ObservedInstallation consistent with the active Install Level's LockfilePinnedConfig?
```

It should report Installed Artifacts, **Broken Symlinks**, **Unexpected Symlink Targets**, missing Managed Skill Content, **Stale Installed Artifacts**, **Unsatisfied Discovery Requirements**, config/lock mismatch, manifest readability, and informational **Unmanaged Artifacts** in configured Client Discovery Locations.

`doctor` answers:

```text
Is my environment/config/tooling capable of working?
```

It should check git availability, Project Root detection, supported clients, path writability, config schema validity, optional network/Skill Source checks, Client Discovery Location confidence warnings, and **Unmanaged Artifacts** only when they block previewed Client Discovery Location paths. It should not replace `status` for ObservedInstallation consistency.

## MVP Acceptance Criteria

- Automated tests pass.
- `agentcfg init` creates `.agentcfg/config.toml`.
- `agentcfg init --project` creates `agentcfg.toml`.
- `agentcfg init --user` creates `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`.
- `agentcfg preview` is read-only.
- `agentcfg apply` resolves and pins a path Skill Source, writes lockfile, materializes Managed Skill Content, and installs an **Installed Artifact** symlink at a Client Discovery Location.
- `agentcfg apply --refresh-sources` performs Source Refresh for path Skill Source content and lockfile.
- `agentcfg prune` removes only manifest-owned **Stale Installed Artifacts** and **Stale Discovery Requirements**.
- Discovery Name Collision handling is tested.
- Internal symlink materialization and external symlink rejection are tested.
- Shared `.agents/skills` Discovery Requirements across Codex/Pi/OpenCode/Cursor are tested.

## Open Product Questions

- Whether git Skill Sources are in the first implementation slice or come after path Skill Sources - Answered: YES
- How much Skill Source provenance to expose for Client Discovery Registry decisions.
- Whether both `skills/<name>/SKILL.md` and root-level `<name>/SKILL.md` Skill Source layouts should be accepted in V1. — Answered: YES; bounded `discovery_depth` with nested-skill exclusion (`design-v1.md`, M2.1).
