# agentcfg

`agentcfg` is a CLI for managing Agent Configuration as repeatable desired state, starting with skills.

It is designed to consume skills from filesystem or git **Skill Sources**, resolve them into **Locked Desired State** in **Managed State** (Manifest and Managed Skill Content under `.agentcfg/` or user state home), and place **Installed Artifacts** safely at **Client Discovery Locations** for the Project Level and User Level.

## Status

The Cargo workspace builds with the pinned toolchain in [rust-toolchain.toml](rust-toolchain.toml).

| Area | State |
| --- | --- |
| CLI surface | `init`, `preview`, `apply`, `prune`, `status`, `doctor` wired to core workflows |
| Config | TOML parse/validate for Skill Sources, Skill Selection, Skill Groups, Skill Aliases, clients |
| Paths | Config layers, lockfiles, Managed State paths, project root discovery |
| Path Skill Source discovery | Discovers `SKILL.md` directories with bounded depth, relative path resolution, hidden/symlink traversal rules, duplicate Source Skill Name diagnostics |
| Skill Selection | Resolves path Skill Source includes and Skill Groups from strict root `skills.toml` metadata |
| Discovery registry | Built-in Client Discovery Locations (shared `.agents/skills` grouping) |
| `init` | Creates config; reports **Unmanaged Artifacts** and scan failures; does not write Client Discovery Locations |
| `preview`, `apply`, `prune`, `status`, `doctor` | Not implemented yet — commands return `unsupported feature` (exit 1) instead of silent success |

Planned soon: Skill Aliases and Discovery Names (implementation plan M2.4), then desired state, preview operations, apply, manifest, and status.

## Prerequisites

```sh
rustup show active-toolchain   # should match rust-toolchain.toml
cargo test --workspace
```

## Goals

- Keep Skill Configuration handling separate from Skill Sources.
- Support skills in **Agent Skill Format** (`SKILL.md` directories).
- Support path and git **Skill Sources**.
- Provide repeatable `preview`, `apply`, and `prune` workflows.
- Manage Shared Project Config, User Project Config, and User Config.
- Install only manifest-owned Installed Artifacts and prune conservatively.
- Prefer portable Client Discovery Locations where Clients support them.

## Non-goals for V1

- Commands, workflows, rules, or MCP management.
- External Skill catalog publishing.
- Desktop UI.
- Arbitrary org/team discovery layers.
- A generic platform for every agent-facing configuration type.

## Commands

```sh
agentcfg init [--project|--user]
agentcfg preview [--user] [--refresh-sources]
agentcfg apply [--user] [--refresh-sources]
agentcfg prune [--user]
agentcfg status [--user]
agentcfg doctor
```

`preview` is read-only once implemented. `apply` installs **Locked Desired State** into Managed State and Client Discovery Locations. `prune` removes **Stale Installed Artifacts** and **Stale Discovery Requirements** from Managed State only. `status` reports managed install-state consistency. `doctor` checks environment and configuration readiness.

Repeatable `--client` for `preview`, `apply`, `prune`, and `status` is specified in the PRD and planned in the implementation plan; it is not in the CLI yet.

## Concepts → code

| Term | Where |
| --- | --- |
| Config Layer, Persisted Scope Value | `crates/agentcfg-core/src/layer_level.rs`, `config.rs`, `config_paths.rs` |
| Install Level | `layer_level.rs`, `workflow` requests, CLI `--user` |
| Skill Source / Selection / Alias (config) | `config.rs`; resolution → `skill_source/` |
| Client Discovery Location / Registry | `discovery_registry.rs`, `workflow::init` |
| Managed State paths | `config_paths.rs` (`ManagedStatePaths`) |
| Desired State / Configured Item | `desired_state.rs` |
| Lockfile / Manifest | `lockfile.rs`, `manifest.rs` |
| Preview / Apply | `workflow` (stubs until later milestones) |

Domain vocabulary: [UBIQUITOUS-LANGUAGE.md](UBIQUITOUS-LANGUAGE.md). Agent policy: [AGENTS.md](AGENTS.md) ([CLAUDE.md](CLAUDE.md) points to the same file).

## Config Layers

- `agentcfg.toml` / `agentcfg.lock` for Shared Project Config.
- `.agentcfg/config.toml` / `.agentcfg/lock.toml` for User Project Config.
- `${XDG_CONFIG_HOME:-~/.config}/agentcfg/config.toml` / `lock.toml` for User Config.

Project Level apply reads Shared Project Config and User Project Config. User Level apply is separate and installs only to user-level Client Discovery Locations.

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

V1 separates Skill Source resolution from Client Discovery Location installation:

```text
Skill Source -> Managed Skill Content -> Client Discovery Location symlink
```

This lets normal `apply` reinstall the locked version without depending on mutable path Skill Sources or floating git refs, while `apply --refresh-sources` performs **Source Refresh** to refresh Skill Source resolutions before applying Locked Desired State.

Cleanup safety rules:

- Remove only manifest-owned Installed Artifacts.
- Refuse **Unexpected Symlink Target** destinations.
- Never delete unmanaged real files.
- Delete directories only when empty and manifest-owned.

## Documentation

- [Ubiquitous language](UBIQUITOUS-LANGUAGE.md)
- [PRD](docs/prd.md)
- [V1 design](docs/design-v1.md)
- [V1 implementation plan](docs/implementation-plan-v1.md)
- [Agent instructions](AGENTS.md)

## License

BSD 3-Clause. See [LICENSE](LICENSE).
