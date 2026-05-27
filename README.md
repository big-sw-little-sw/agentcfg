# agentcfg

`agentcfg` is a CLI for managing agent configuration as repeatable desired state, starting with skills.

It is designed to consume skills from filesystem or git sources, resolve them into **Locked Desired State** in **Managed State** (Manifest and Managed Skill Content under `.agentcfg/` or user state home), and install client-specific skill entrypoints safely at the Project Level and User Level.

## Status

Early implementation stage. The repository contains the V1 PRD and design notes, plus the initial Cargo workspace, CLI command surface, and core workflow stubs.

## Goals

- Keep the skill manager separate from skill repositories.
- Support skills in **Agent Skill Format** (`SKILL.md` directories).
- Support path and git **Skill Sources**.
- Provide repeatable `preview`, `apply`, and `prune` workflows.
- Manage shared project, user project, and user-level configuration.
- Install only manifest-owned Installed Artifacts and prune conservatively.
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
agentcfg preview [--user] [--refresh-sources]
agentcfg apply [--user] [--refresh-sources]
agentcfg prune [--user]
agentcfg status [--user]
agentcfg doctor
```

`preview` is read-only. `apply` installs **Locked Desired State** into Managed State and Client Discovery Locations. `prune` removes stale manifest-owned Installed Artifacts only.

## Configuration Layers

- `agentcfg.toml` / `agentcfg.lock` for shared project configuration.
- `.agentcfg/config.toml` / `.agentcfg/lock.toml` for user project configuration.
- `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml` / `lock.toml` for user configuration.

Project apply reads shared and user project config. User apply is separate and installs only to user-level Client Discovery Locations.

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

V1 separates source acquisition from Client Discovery Location installation:

```text
Skill Source -> Managed Skill Content -> Client Discovery Location symlink
```

This lets normal `apply` reinstall the locked version without depending on mutable path sources or floating git refs, while `apply --refresh-sources` performs **Source Refresh** to refresh Skill Source resolutions before applying Locked Desired State.

Cleanup safety rules:

- Remove only manifest-owned Installed Artifacts.
- Refuse unexpected symlink targets.
- Never delete unmanaged real files.
- Delete directories only when empty and manifest-owned.

## Documentation

- [PRD](docs/prd.md)
- [V1 design](docs/design-v1.md)

## License

BSD 3-Clause. See [LICENSE](LICENSE).
