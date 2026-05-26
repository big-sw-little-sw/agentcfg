# agentcfg

`agentcfg` is a CLI for managing agent configuration as repeatable desired state, starting with skills.

It is designed to consume skills from filesystem or git sources, resolve them into locked managed state, and install client-specific skill entrypoints safely across project and user install scopes.

## Status

Early implementation stage. The repository contains the V1 PRD and design notes, plus the initial Cargo workspace, CLI command surface, and core workflow stubs.

## Goals

- Keep the skill manager separate from skill repositories.
- Support standard `SKILL.md` skill directories.
- Support path and git skill sources.
- Provide repeatable `plan`, `sync`, and `prune` workflows.
- Manage shared project, user project, and user-level configuration.
- Install only manifest-owned artifacts and prune conservatively.
- Prefer portable skill paths where clients support them.

## Non-goals for V1

- Commands, workflows, rules, or MCP management.
- Skill registry publishing.
- Desktop UI.
- Arbitrary org/team discovery layers.
- A generic cross-agent configuration platform.

## Planned commands

```sh
agentcfg init [--project|--user]
agentcfg plan [--user] [--upgrade]
agentcfg sync [--user] [--upgrade]
agentcfg prune [--user]
agentcfg status [--user]
agentcfg doctor
```

`plan` is read-only. `sync` installs the locked desired state. `prune` removes stale manifest-owned artifacts only.

## Configuration Layers

- `agentcfg.toml` / `agentcfg.lock` for shared project configuration.
- `.agentcfg/config.toml` / `.agentcfg/lock.toml` for user project configuration.
- `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml` / `lock.toml` for user configuration.

Project sync reads shared and user project config. User sync is separate and installs only to user-level targets.

## Supported clients planned for V1

| Client | Project skills | User skills |
| --- | --- | --- |
| Codex | `.agents/skills/{name}` | `~/.agents/skills/{name}` |
| Pi | `.agents/skills/{name}` | `~/.agents/skills/{name}` |
| OpenCode | `.agents/skills/{name}` | `~/.agents/skills/{name}` |
| Claude Code | `.claude/skills/{name}` | `~/.claude/skills/{name}` |
| Cline | `.cline/skills/{name}` | `~/.cline/skills/{name}` |
| Cursor | `.agents/skills/{name}` | `~/.agents/skills/{name}` |

## Design notes

V1 separates source acquisition from target installation:

```text
source -> managed materialized tree -> client target symlink
```

This lets normal `sync` reinstall the locked version without depending on mutable path sources or floating git refs, while `sync --upgrade` can refresh source resolutions explicitly.

Cleanup safety rules:

- Remove only manifest-owned artifacts.
- Refuse unexpected symlink targets.
- Never delete unmanaged real files.
- Delete directories only when empty and manifest-owned.

## Documentation

- [PRD](docs/prd.md)
- [V1 design](docs/design-v1.md)

## License

BSD 3-Clause. See [LICENSE](LICENSE).
