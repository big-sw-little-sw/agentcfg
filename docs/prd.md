# agentcfg PRD

## Purpose

`agentcfg` is a CLI for managing agent configuration as repeatable desired state, starting with skills.

V1 should be source-agnostic, explicit about lifecycle state, and compatible with the emerging `SKILL.md` Agent Skills ecosystem. It should manage skills well without becoming a full cross-agent configuration platform.

Detailed persisted contracts and safety rules live in [design-v1.md](design-v1.md).

## Goals

- Separate the skill manager from skill repositories.
- Consume skills from filesystem path and git sources.
- Keep skills in the standard `SKILL.md` directory format.
- Support repeatable `plan` and `sync` behavior.
- Support shared project, personal project, and user configuration scopes.
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
- **Target**: a client-specific filesystem location where `agentcfg` installs a skill entrypoint, such as `.agents/skills/{name}`.
- **Source**: a filesystem path or git location containing skill directories.
- **Managed source tree**: a generated copy of resolved skill content under `agentcfg` state. Targets point to this tree so normal sync can install the locked version without depending on a mutable source path or moving git branch.
- **Config layer**: one active config file participating in planning or sync, such as shared project config or personal project config.
- **Install scope**: whether a command installs or inspects project-level targets or user-level targets.
- **Consumer**: a `{scope, client}` pair recorded in the manifest to say which config/client consumes an installed target artifact. A shared target can be pruned only when it has no remaining consumers.

## Config Types

V1 has three user-facing config types:

- **Shared project config**: `agentcfg.toml` at the repo root. This is committed when a repo intentionally wants common agent skills for everyone working in that project. Its lockfile is `agentcfg.lock`.
- **Personal project config**: `.agentcfg/config.toml` inside a repo. This is for one user's additions in that specific project and should stay uncommitted with the rest of `.agentcfg/`. Its lockfile is `.agentcfg/lock.toml`.
- **User config**: `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`. This is for one user's skills across projects and syncs only to user-level client targets. Its lockfile is `${XDG_CONFIG_HOME:-~/.config}/agentcfg/lock.toml`.

Project sync reads shared project config plus personal project config. User config is synced separately with `agentcfg sync --user`; it is not merged into project sync.

Rationale: agent clients already define how user-level and project-level skills are combined at runtime. `agentcfg` should install user config to user targets and project config to project targets, then let each client apply its own merge and precedence rules. This avoids duplicating user skills into every project and avoids second-guessing client-specific behavior.

## Commands

V1 commands:

```bash
agentcfg init
agentcfg init --project
agentcfg init --user

agentcfg plan
agentcfg plan --upgrade
agentcfg plan --user
agentcfg plan --user --upgrade

agentcfg sync
agentcfg sync --upgrade
agentcfg sync --user
agentcfg sync --user --upgrade

agentcfg prune
agentcfg prune --user
agentcfg status
agentcfg status --user
agentcfg doctor
```

Command semantics:

- `init`: create config for the selected config layer. Default creates personal project config at `.agentcfg/config.toml`.
- `init --project`: create shared project config at `agentcfg.toml`.
- `init --user`: create user config at `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`.
- `plan`: strict read-only preview. No persistent writes to config, lockfiles, manifests, sources, caches, or targets.
- `plan --upgrade`: read-only preview after refreshing source resolutions in memory. For git sources, this means checking whether floating refs moved. For path sources, this means checking whether source content changed.
- `sync`: create missing lockfiles if needed, then install the locked resolved state.
- `sync --upgrade`: refresh source resolutions, update active lockfiles, materialize refreshed managed source trees, then install.
- `prune`: remove stale managed artifacts and stale consumers. It applies by default because `plan` is the preview command.
- `status`: inspect current managed install state.
- `doctor`: diagnose environment, client support, config validity, path writability, and optional network/source issues.
- `--user`: use user config and user targets for `plan`, `sync`, `prune`, and `status`. It selects the user install scope for those commands and is not needed for `doctor`, which is diagnostic.

## User Workflows

Personal project setup:

```bash
agentcfg init
agentcfg plan
agentcfg sync
```

Shared project setup:

```bash
agentcfg init --project
agentcfg plan
agentcfg sync
```

User-level setup:

```bash
agentcfg init --user
agentcfg plan --user
agentcfg sync --user
```

Upgrade selected sources:

```bash
agentcfg plan --upgrade
agentcfg sync --upgrade
```

Remove stale managed artifacts:

```bash
agentcfg plan
agentcfg prune
```

## Plan, Sync, and Prune

`plan` should show:

- lockfile changes that would be created or updated
- source resolutions
- skills to create
- skills to update
- consumer additions
- stale consumer removals
- stale artifact removals
- alias rewrites
- warnings for uncertain client targets

`sync` applies:

- missing lockfile creation
- target creates
- target updates
- consumer additions

`sync` does not remove stale artifacts. If stale artifacts remain, it warns:

```text
Stale managed artifacts remain: N
These may still be discovered by agent clients until pruned.
Run: agentcfg prune
```

`prune` applies:

- stale consumer removals
- stale managed artifact removals

Cleanup safety invariants:

- Remove only manifest-owned artifacts.
- Refuse unexpected symlink targets.
- Never delete unmanaged real files.
- Delete directories only if empty and manifest-owned.

## Supported Clients

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

Known native alternatives are design details covered in [design-v1.md](design-v1.md).

## Status and Doctor

`status` answers:

```text
Is the current managed install state consistent?
```

It should report installed managed artifacts, broken symlinks, unexpected symlink targets, missing managed sources, stale managed artifacts, config/lock mismatch, manifest readability, and informational unmanaged artifacts in configured target directories.

`doctor` answers:

```text
Is my environment/config/tooling capable of working?
```

It should check git availability, repo root detection, supported clients, path writability, config schema validity, optional network/source checks, target confidence warnings, and unmanaged artifacts only when they block planned target paths.

## MVP Acceptance Criteria

- Automated tests pass.
- `agentcfg init` creates `.agentcfg/config.toml`.
- `agentcfg init --project` creates `agentcfg.toml`.
- `agentcfg init --user` creates `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml`.
- `agentcfg plan` is read-only.
- `agentcfg sync` resolves a path skill source, writes lockfile, materializes managed source, and installs skill symlink.
- `agentcfg sync --upgrade` refreshes path source content and lockfile.
- `agentcfg prune` removes only manifest-owned stale artifacts.
- Alias collision handling is tested.
- Internal symlink materialization and external symlink rejection are tested.
- Shared `.agents/skills` consumers across Codex/Pi/OpenCode/Cursor are tested.

## Open Product Questions

- Whether git sources are in the first implementation slice or come after path sources - Answered: YES
- How much source provenance to expose for client target registry decisions.
- Whether both `skills/<name>/SKILL.md` and root-level `<name>/SKILL.md` source layouts should be accepted in V1.
