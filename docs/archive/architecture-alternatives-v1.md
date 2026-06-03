# agentcfg V1 Architecture Alternatives

This note records three clean-slate V1 architecture alternatives. It is an
alternatives document, not the implementation plan of record.

Inputs:

- [CONTEXT.md](../CONTEXT.md)
- [prd.md](prd.md)
- [design-v1.md](design-v1.md)
- [implementation-plan-v1.md](implementation-plan-v1.md)
- `do-improve-codebase-architecture` language: Module, Interface,
  Implementation, Depth, Seam, Adapter, Leverage, and Locality

The subagent review loop produced four lenses:

- product and glossary semantics
- module shape and deepening opportunities
- safety and persistence
- post-V1 extensibility to other Configured Item kinds

## Shared Invariants

All three alternatives must preserve these invariants.

- V1 manages Skill Configuration only.
- Preview is read-only. It does not write config, Lockfiles, the Manifest,
  Managed State, Skill Sources, or Client Discovery Locations.
- Apply is one-way from Skill Source to Managed Skill Content to Client
  Discovery Location. Apply never writes Skill Sources.
- Prune is the only workflow that removes stale Managed State.
- Lockfile is the repeatability authority for Locked Desired State.
- Manifest is the ownership authority for Installed Artifacts and Discovery
  Requirements.
- Client Discovery Registry is the location authority for Client Discovery
  Locations.
- Config Layer, Install Level, and Persisted Scope Value stay distinct.
- Project Level uses Shared Project Config plus User Project Config. User Level
  uses User Config only.
- Config Layers are additive in V1. There is no silent replacement or precedence
  policy.
- `--client` narrows configured Clients. It does not add a Client outside active
  configuration, except that `clients = "all"` means supported Clients at the
  selected Install Level.
- Skill Alias changes Discovery Name and does not mutate Skill Sources.
- Discovery Name Collision is detected per Client Discovery Location.
- Unmanaged Artifacts are never overwritten or deleted.
- Unexpected Symlink Target blocks update and prune.
- External symlinks and special files in Skill Sources are rejected during
  materialization.
- CLI command handlers are adapters into core workflow Modules.

## Recommendation Summary

| Alternative | Core bet | Recommendation |
| --- | --- | --- |
| Alt-1: Skill-first, shared operations | Skill-specific resolution is the deep V1 Module; shared operation planning begins after Desired State records exist. | Best primary V1 shape. |
| Alt-2: Layer Contract Compiler | Config Layer plus Lockfile compilation is the deep Module; Installed Artifacts and operations are projections from that contract. | Strong for clean-slate persistence correctness, but wider and slower to execute. |
| Alt-3: Workflow Kernel | Preview, Apply, Prune, and Status are transitions over one install-state machine. | Useful refinement for safety, but too broad as the central V1 shape. |

The strongest path is Alt-1 with two ideas borrowed from the others:

- from Alt-2: keep Config Layer provenance explicit everywhere Desired State,
  Lockfile, Manifest, and Discovery Requirement records cross a seam
- from Alt-3: use workflow-level read/write gating so Preview cannot mutate by
  construction

Detailed design files:

- [Alt-1: Skill-First Resolution, Shared Operations](architecture-alt-1-skill-first-shared-operations.md)
- [Alt-2: Layer Contract Compiler](architecture-alt-2-layer-contract-compiler.md)
- [Alt-3: Workflow Kernel](architecture-alt-3-workflow-kernel.md)
- [Recommended V1 Architecture](architecture-recommendation-v1.md)

## Alt-1: Skill-First Resolution, Shared Operations

### Thesis

Alt-1 keeps V1 concrete: Skill Configuration is parsed and resolved by
skill-specific Modules. After Skill Source resolution, Skill Selection, Skill
Alias handling, Managed Skill Content preparation, and hashing, the resolver
emits kind-aware Desired Installed Artifact records.

The shared Module starts there:

```text
skill-specific resolution
  -> kind-aware Desired Installed Artifact records
  -> shared operation planning
  -> Preview / Apply / Prune / Status execution
```

V1 uses `kind = "skill"` as data in Desired State, Manifest, operation records,
and Client Discovery Registry lookup keys. It does not introduce generic
Configured Item manager traits, factories, or registries.

### Module Shape

| Module | Interface | Ownership |
| --- | --- | --- |
| `workflow` | `init`, `preview`, `apply`, `prune`, `status`, `doctor` request/result functions | Thin orchestration into deeper Modules. |
| `install_session` | `build_session(InstallLevel)` | Active Config Layers, Project Root, User paths, config paths, Lockfile paths, Managed State roots. |
| `skill_config` | load and validate active Skill Configuration layers | TOML schema, Persisted Scope Value validation, skill-specific config shape. |
| `skill_source` | resolve Skill Sources with `UseLocked` or `RefreshSources` | Path and git Skill Source acquisition, discovery, Skill Group metadata. |
| `skill_selection` | selected skills with Source Skill Name and Discovery Name | Included Skills, Skill Groups, Skill Aliases, deterministic ordering. |
| `skill_content` | prepared skill tree plus hashes | Safe tree walk, Discovery Name preparation, external symlink rejection, installed hash. |
| `lockfile` | read/write Locked Desired State per Config Layer | Deterministic Lockfile persistence only. |
| `managed_skill_content` | ensure/read content-addressed Managed Skill Content | Writes prepared Skill files under Managed State. |
| `discovery_registry` | resolve configured Clients and Client Discovery Locations | Supported Clients, `clients = "all"`, `--client` narrowing, confidence/provenance. |
| `desired_state` | assemble desired Installed Artifacts and Discovery Requirements | Fan-in from prepared skill facts and resolved Client facts. |
| `manifest` | read/write Manifest records | Installed Artifact ownership and Discovery Requirement persistence. |
| `install_health` | classify current state | Broken Symlink, Unexpected Symlink Target, missing Managed Skill Content, stale and unsatisfied facts. |
| `operations` | build operation plan | Discovery Name Collision, creates, updates, stale reporting, prune candidates. |
| `execution` | execute Apply and Prune operations | Filesystem mutation and Manifest/Lockfile writes with precondition checks. |

### Flow

```text
workflow request
  -> install_session
  -> skill_config
  -> lockfile
  -> skill_source
  -> skill_selection
  -> skill_content
  -> managed_skill_content
  -> discovery_registry
  -> desired_state
  -> install_health
  -> operations
  -> execution only for Apply / Prune
```

### Safety and Persistence

Lockfiles freeze only what Apply needs to repeat:

- Config Layer
- Skill Source id and type
- stable Skill Source locator
- requested git ref and resolved commit, for git
- Source Skill Name
- Discovery Name
- `source_hash`
- installed hash
- materialization version
- Discovery Name preparation state

Lockfiles do not store machine-local Client Discovery Location paths.

Manifest records physical ownership:

- kind, currently `skill`
- Install Level
- Client Discovery Location
- Discovery Name
- expected symlink destination
- installed hash
- Skill Source provenance
- Discovery Requirements

Shared Client Discovery Locations are represented as one Installed Artifact with
many Discovery Requirements.

### Depth and Locality

Alt-1 has good Depth because callers learn small Interfaces:

- resolve skills
- assemble Desired State
- classify install health
- build operations
- execute safe mutations

The deletion test passes when `operations` owns real policy: lockfile/Manifest
comparison, Discovery Requirement merges, stale detection, Discovery Name
Collision reporting, and safe update/prune decisions. It fails if `operations`
is only a pass-through list mapper.

### Post-V1 Fit

Alt-1 reserves a narrow future seam: kind-specific resolvers can later produce
Desired State entries for other Configured Item kinds.

It defers:

- generic Configured Item resolver traits
- generic Configured Item config schema
- assuming every future Configured Item creates symlinked Installed Artifacts
- generic external-origin terms beyond Skill Source
- MCP, rules, and agent-definition conflict rules

### Risks

- The operation plan can become too generic if it tries to model future kinds
  before they exist.
- The skill resolver can become too wide unless Skill Source resolution, Skill
  Selection, Skill Alias handling, and materialization each keep clear Locality.
- Workflow functions must stay adapters, not hidden orchestration piles.

## Alt-2: Layer Contract Compiler

### Thesis

Alt-2 makes Active Config Layers plus adjacent Lockfiles the center of the
design. The deepest Module compiles an Install Level into one canonical internal
contract containing Desired State and Locked Desired State facts. Client
Discovery Locations, Installed Artifacts, install health, and workflow
operations are projections from that contract.

User-facing and persisted language remains glossary-native. "Layer Contract" is
internal architecture language only.

```text
Active Config Layers + Lockfiles
  -> compiled Desired State / Locked Desired State contract
  -> Client Discovery Location projection
  -> Installed Artifact projection
  -> operation projection
```

### Module Shape

| Module | Interface | Ownership |
| --- | --- | --- |
| `install_context` | `resolve(request) -> ActiveInstallContext` | Install Level, Active Config Layers, paths, Managed State roots. |
| `layer_contract` | `compile(context, SourceRefreshPolicy, client_filter)` | Layer order, additive behavior, adjacent Lockfiles, missing/stale lock facts, Source Refresh policy. |
| `skill_layer_adapter` | `resolve_skill_layer(layer, lockfile, policy)` | Skill Source discovery, Skill Selection, Skill Groups, Skill Aliases, Discovery Name preparation, hashing. |
| `client_projection` | `project_clients(layer_contract)` | `clients = "all"`, explicit Clients, `--client` narrowing, Client Discovery Registry lookup. |
| `desired_state_compiler` | `compile(layer_contract, skill_facts, discovery_projection)` | Discovery Requirements, shared Client Discovery Location aggregation, Discovery Name Collision, desired Installed Artifact identity. |
| `install_health` | `classify(desired_state, manifest, filesystem)` | Manifest/filesystem consistency facts. |
| `operation_projection` | `preview`, `apply`, `prune`, `status` projections | Structured workflow output and safe mutation from compiled facts. |
| `workflow` | request/result entrypoints | Thin orchestration. |
| `agentcfg-cli` | parsing and rendering | Terminal behavior only. |

The main seam is between `layer_contract` and its private kind-specific Adapter.
In V1 that Adapter is only `skill_layer_adapter`, so the seam stays private.

### Flow

```text
workflow request
  -> install_context
  -> layer_contract
       -> loads Active Config Layers
       -> loads adjacent Lockfiles
       -> applies UseLocked or RefreshSources
       -> calls skill_layer_adapter
  -> client_projection
  -> desired_state_compiler
  -> install_health
  -> operation_projection
```

### Safety and Persistence

Alt-2 gives strong Locality to persistence rules:

- Lockfile adjacency and write policy live in `layer_contract`.
- Config Layer provenance is present before any Desired State projection.
- Discovery Requirements are compiled before Installed Artifacts are projected.
- Discovery Name Collision is rejected before filesystem mutation.
- Shared Client Discovery Locations become one Installed Artifact with many
  Discovery Requirements.
- Apply and Prune act from compiled Desired State plus Manifest ownership, not
  path scans alone.

Plain Apply uses Locked Desired State. Source Refresh is the only workflow path
that changes lock facts.

### Depth and Locality

The `layer_contract` Module is deep if its Interface is effectively "compile
this Install Level" and its Implementation hides layer ordering, Lockfile reuse,
Source Refresh, client filtering, requirement aggregation, and collision safety.

It is shallow if it becomes a bag of schema structs and pushes Skill Selection,
Discovery Name preparation, and Client Discovery Registry policy back to
callers.

### Post-V1 Fit

Alt-2 is attractive if later kinds make Config Layer composition the hardest
problem. MCP may need layer-sensitive secrets and environment references. Rules
may need shared vs user additions to be very explicit. Agent definitions may
care about Project Level versus User Level composition.

It should still defer:

- a generic `[configured_items]` config shape
- cross-kind precedence or replacement semantics
- shared conflict rules across unrelated Configured Item kinds
- public layer validation hooks before a second kind proves the shape

### Risks

- It can over-center persisted contracts and make skill behavior look like
  validation wrapped around TOML.
- `desired_state_compiler` can become wide.
- It slows execution because layer and Lockfile semantics must become deep
  before operation work can progress.
- It is not the best match for the current implementation plan, which already
  has skill-first progress.

## Alt-3: Workflow Kernel State Machine

### Thesis

Alt-3 makes workflow transitions the deepest Module. Preview, Apply, Prune, and
Status are not separate orchestrations that share helpers. They are allowed
transitions over the same install-state machine.

```text
workflow request
  -> install-state machine
  -> transition set
  -> optional commit
```

Desired State, install health, operation planning, and commit behavior are
internal phases. Callers ask for workflow outcomes, not intermediate products.

### Module Shape

| Module | Interface | Ownership |
| --- | --- | --- |
| `workflow` | public request/result functions | CLI adapter entrypoint only. |
| `workflow_kernel` | `preview`, `apply`, `prune`, `status` | State machine, phase ordering, command invariants, read/write rules. |
| `workflow_kernel::session_phase` | internal `InstallSession` | Install Level, Active Config Layers, config paths, Lockfile paths, Managed State roots. |
| `workflow_kernel::skill_phase` | internal `ResolvedSkillIntent` | Skill Source discovery, Skill Selection, Skill Groups, Skill Alias, Discovery Name facts. |
| `workflow_kernel::managed_state_phase` | internal `ManagedContentIntent` | Managed Skill Content paths, materialization intent, hashes, Lockfile reconciliation. |
| `workflow_kernel::discovery_phase` | internal `ResolvedDiscoveryLocations` | Client Discovery Registry lookup, `clients = "all"`, `--client` narrowing. |
| `workflow_kernel::manifest_phase` | internal `ManifestSnapshot` | Manifest read/write model, Installed Artifact records, Discovery Requirements. |
| `workflow_kernel::health_phase` | internal `InstallSnapshot` | Unmanaged Artifact, Broken Symlink, Unexpected Symlink Target, stale and unsatisfied facts. |
| `workflow_kernel::transition_phase` | internal `TransitionSet` | Discovery Name Collision, creates, updates, stale reporting, prune candidates. |
| `workflow_kernel::commit_phase` | internal `CommitPlan` | Apply and Prune writes with preconditions. |

### Flow

```text
CLI request
  -> workflow adapter
  -> workflow_kernel opens InstallSession
  -> load Active Config Layers, Lockfiles, Manifest
  -> resolve skills into Discovery Name-bearing intent
  -> reconcile Locked Desired State and Managed Skill Content intent
  -> resolve Clients and Client Discovery Locations
  -> classify Manifest / Managed State / Client Discovery Locations
  -> build TransitionSet
  -> Preview renders TransitionSet
  -> Apply commits allowed apply transitions
  -> Prune commits allowed prune transitions
  -> Status reports InstallSnapshot consistency
```

Preview and Apply share the same path until commit. Preview has no mutating
Adapter available, so read-only behavior is structural. Prune is not Apply with
deletes enabled; it is a separate transition over stale Manifest facts.

### Safety and Persistence

Alt-3 is strongest at making mutation rules explicit:

- TransitionSet carries preconditions such as expected symlink destination,
  expected Manifest record, no Unmanaged Artifact at the path, directory-empty
  checks, and Managed Skill Content existence.
- `commit_phase` rechecks preconditions before writing.
- Apply may create/update Installed Artifacts, add Discovery Requirements, write
  Lockfiles, materialize Managed Skill Content, and update Manifest records.
- Apply never removes stale state.
- Prune acts only from Manifest-owned records plus current filesystem checks.
- Status consumes the same InstallSnapshot that Apply and Prune trust.

### Depth and Locality

The public `workflow_kernel` Interface is small. Its Implementation is large.
That can be deep if callers truly do not need to know phase ordering or safety
rules.

The risk is that the Interface becomes deceptively small while every test and
future change needs to understand internal phases. That would violate
"the Interface is the test surface" by pushing tests into internals.

### Post-V1 Fit

Alt-3 becomes more attractive after several Configured Item kinds exist and
Apply may need native config-file edits rather than only symlinked Installed
Artifacts.

For V1, it should defer:

- resolver/applier/status Adapter Interfaces
- dynamic registration of Configured Item kinds
- cross-kind transaction semantics
- rollback semantics for native config-file edits

### Risks

- It is likely too wide for V1.
- The kernel seam has only one real kind, Skill, so most Configured Item seams
  would be hypothetical.
- Future kinds must be added as internal phases instead of independent producers
  of Desired State entries.
- It can blur glossary workflow meanings if every command is treated as a
  generic transition too early.

## Glossary Impact

No V1 glossary update is required for these alternatives.

Post-V1 candidates, only after corresponding designs exist:

- MCP Server
- Rule
- Agent Definition
- Configured Item selector
- kind-specific native config edit terms, if MCP or rules require them

Do not add a generic external-origin term in V1. Use Skill Source until multiple
Configured Item kinds prove they share the same external-origin lifecycle.

## Final Evaluation

Alt-1 is the best V1 design. It preserves the current PRD and glossary, matches
the existing V1 design direction, and creates real Depth without speculative
generic machinery.

Alt-2 is a viable clean-slate alternative if the team decides Lockfile and
Config Layer correctness should dominate implementation shape from the start.
Its useful contribution is stronger Config Layer provenance and persistence
Locality.

Alt-3 is not recommended as the primary V1 design. Its useful contribution is
explicit transition preconditions for Apply and Prune. Those ideas should be
borrowed inside Alt-1's execution path rather than promoted into the central
architecture.
