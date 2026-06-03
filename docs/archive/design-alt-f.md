# Alt-F: Contract-First Domain Kernel

Input boundary: this alternative was derived from `docs/prd.md` and the requested architecture/maintainability/design-review principles only. It intentionally does not use existing design files, alternate designs, or repository code.

## Thesis

V1 is implemented around a contract-first domain kernel. Durable PRD terms such as Config Layer, Install Level, Desired State, Locked Desired State, Manifest, Discovery Requirement, Installed Artifact, Managed Skill Content, and Client Discovery Location become explicit invariant-bearing types first. Each command is a named use case over those contracts. Filesystem, git, TOML, symlink, and environment operations are narrow adapters.

This is not a workflow pipeline and not a reconciler/controller. There is no generic step runner and no desired/current control loop. Each command has a fixed transaction shape, and shared behavior is pulled into deep domain modules only when it protects invariants or reduces caller knowledge.

## Module Map

- `cli`: argument parsing, request construction, terminal rendering, and exit codes only.
- `domain`: invariant-bearing types for Config Layer, Install Level, Skill Source, Skill Selection, Skill Alias, Discovery Name, Locked Desired State, Manifest, Installed Artifact, Discovery Requirement, and install-health findings.
- `use_cases`: one explicit module per command: `init`, `preview`, `apply`, `prune`, `status`, and `doctor`.
- `desired_state`: compiles active Config Layers into Desired State, applies client filters, expands selections/groups/aliases, and detects Discovery Name Collisions.
- `source_resolution`: resolves path and git Skill Sources into source snapshots and skill inventories.
- `locking`: compares Desired State with lockfiles and produces Locked Desired State plus lockfile deltas.
- `materialization`: prepares Managed Skill Content from Locked Desired State, including alias/frontmatter preparation and symlink policy.
- `install_contract`: computes creates, updates, Discovery Requirement additions, stale requirements, stale artifacts, and install-state findings.
- `safety`: owns cleanup refusal rules: manifest ownership, unexpected symlink targets, unmanaged artifacts, empty directory deletion, and external symlink rejection.
- `repositories`: narrow persistence interfaces such as `ConfigRepo`, `LockRepo`, `ManifestRepo`, `ManagedContentRepo`, `DiscoveryRepo`, `SourceRepo`, and `EnvironmentProbe`.
- `adapters`: TOML, filesystem, git, symlink, and client-location implementations.

The deepest modules are `domain`, `desired_state`, `locking`, `install_contract`, and `safety`. Their interfaces are the test surface. Use cases should not know low-level filesystem rules, lockfile repeatability mechanics, or symlink cleanup policy.

## Command Flow

`init`

1. Resolve the selected Config Layer.
2. Ask `domain` for the default config shape.
3. Write only that config through `ConfigRepo`.
4. Do not resolve sources, create lockfiles, touch Managed State, or inspect Client Discovery Locations.

`preview`

1. Load active config and lockfiles.
2. Resolve sources in memory.
3. With `--refresh-sources`, calculate refreshed source snapshots without writing.
4. Compile Desired State.
5. Produce Locked Desired State and lockfile deltas.
6. Inspect manifest, managed content, and discovery locations read-only.
7. Return a preview report: lockfile changes, source resolutions, artifact creates/updates, Discovery Requirement additions/removals, stale artifacts, Discovery Name preparation, and warnings.
8. Enforce read-only behavior with a read-only repository bundle.

`apply`

1. Load config and current locks.
2. Resolve sources, refreshing if requested.
3. Produce Locked Desired State.
4. Write missing or updated lockfiles as needed.
5. Materialize Managed Skill Content.
6. Create/update Installed Artifacts at Client Discovery Locations.
7. Add Discovery Requirements to the Manifest.
8. Warn about stale requirements/artifacts, but never remove them.

`prune`

1. Load active Desired State and Manifest.
2. Classify stale Discovery Requirements and stale Installed Artifacts.
3. Execute only safety-approved removals.
4. Remove manifest records after corresponding cleanup succeeds.
5. Never perform install/update work.

`status`

1. Load config, lockfiles, manifest, managed content, and discovery locations.
2. Report whether managed install state is consistent.
3. Classify broken symlinks, unexpected targets, missing managed content, stale artifacts, unsatisfied requirements, config/lock mismatch, manifest readability, and unmanaged artifacts.

`doctor`

1. Check environment and configuration readiness.
2. Validate git availability, Project Root detection, supported clients, path writability, config schema, optional source/network checks, and uncertain Client Discovery Locations.
3. Inspect unmanaged artifacts only when they block previewed paths.
4. Do not report managed install-state consistency.

## Persistence Model

- Config files are user-authored TOML per Config Layer.
- Lockfiles live beside their config files and store Locked Desired State for repeatable source resolution.
- Managed State owns the Manifest plus Managed Skill Content.
- Manifest records Installed Artifacts and Discovery Requirements keyed by Config Layer, Client, and Install Level.
- Managed Skill Content is prepared from locked source snapshots, not live source paths.
- Client Discovery Locations expose managed content through manifest-owned symlinks or copies.
- Repositories are persistence authorities; domain modules never perform direct filesystem or git operations.

## Safety Invariants

- `preview` performs no persistent writes.
- `apply` does not prune.
- `prune` removes only manifest-owned artifacts.
- Unmanaged real files are never deleted.
- Unexpected symlink targets are refused.
- Broken symlinks are reported before unsafe mutation.
- Directories are deleted only when empty and manifest-owned.
- Discovery Name Collisions fail before install.
- `--client` can narrow configured clients, never add unconfigured clients unless config uses `clients = "all"`.
- User-level and project-level apply remain separate.
- External source symlinks are rejected or safely materialized according to policy.
- `status` and `doctor` answer different questions and must not collapse into one command.

## Testing Strategy

- Pure domain tests for Desired State compilation, client filtering, aliasing, collision detection, stale classification, and status findings.
- Locking tests for repeatability, refresh deltas, config/lock mismatch, and path source content changes.
- Safety tests for manifest-owned cleanup, unmanaged files, unexpected symlink targets, broken symlinks, empty directory deletion, internal symlink materialization, and external symlink rejection.
- Use-case tests with fake repositories to prove command write sets: preview writes nothing, apply writes locks/content/artifacts/manifest only, and prune deletes only stale owned state.
- Temp-filesystem integration tests for path Skill Source apply, refresh apply, prune, and shared `.agents/skills` Discovery Requirements across multiple clients.
- CLI golden tests only for stable user-facing summaries and errors.

## Design Risks

- Command use cases may duplicate setup logic. Keep shared derivation in deep modules like `desired_state`, `locking`, and `install_contract`, not in a generic workflow runner.
- Repository traits can become ceremony. Keep them narrow and persistence-owned; do not invent broad storage abstractions.
- Source resolution can leak git/path complexity. Hide it behind `source_resolution` and return domain snapshots, not adapter details.
- Client support can become a platform. Keep V1 built-ins as data plus validation, not a plugin system.
- Materialization can become policy-heavy. Concentrate alias/frontmatter/symlink handling in `materialization` plus `safety`.

## What Not To Abstract

- Do not abstract Configured Item kinds beyond Skill.
- Do not build a generic reconciler, controller loop, task graph, or workflow engine.
- Do not abstract client precedence or runtime merge behavior.
- Do not introduce org/team layer discovery.
- Do not build a catalog or plugin model for Skill Sources.
- Do not generalize every filesystem operation behind broad traits.
- Do not add flags or modes for cleanup beyond preview/apply/prune separation.
- Do not split tiny pass-through modules unless they protect invariants or reduce caller knowledge.
