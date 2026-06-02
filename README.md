# agentcfg

`agentcfg` is a CLI concept for managing Agent Configuration as repeatable desired state, starting with skills.

It is intended to consume skills from filesystem or git **Skill Sources**, resolve them into **Locked Desired State** in **Managed State**, and place **Installed Artifacts** safely at **Client Discovery Locations** for the Project Level and User Level.

## Status

Clean-slate implementation prep is complete: this repository currently contains product/design documentation and agent workflow assets only. The previous Rust workspace, package files, source code, tests, lockfile, toolchain pin, and build artifacts have been removed.

No build or test command is currently defined.

## Documentation

- [Context](CONTEXT.md)
- [PRD](docs/prd.md)
- [V1 design](docs/design-v1.md)
- [Agent instructions](AGENTS.md)

## License

BSD 3-Clause. See [LICENSE](LICENSE).
