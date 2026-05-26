# Agent Configuration Management

Agent Configuration Management defines how agent-facing configuration becomes repeatable, managed install state. V1 is limited to skill configuration; later versions may add other agent configuration items.

## Language

### Agent Configuration and Skills

**Agent Configuration**:
User-authored intent for what agent capabilities should be available. V1 includes **Skill Configuration** only.
_Avoid_: agent config, settings

**Skill Configuration**:
The part of **Agent Configuration** that declares which skills should be available to which agent clients.
_Avoid_: skill setup, skill install config

**Configured Item**:
One kind of agent-facing thing managed by `agentcfg`. V1 has one Configured Item kind: **Skill**.
_Avoid_: resource, object, config item

**Agent Skill Format**:
The `SKILL.md` directory format used by the Agent Skills ecosystem.
_Avoid_: Agent Skills Standard, standard SKILL.md format

**Skill**:
A **Configured Item** in the **Agent Skill Format** that a **Client** can discover and use.
_Avoid_: plugin, capability, tool

**Skill Source**:
A filesystem path or git location that contains one or more **Skills** available for selection.
_Avoid_: Source, repository, source tree

**Managed Skill Content**:
The `agentcfg`-owned skill files prepared from Locked Desired State and stored in managed state. Managed Skill Content is derived from a **Skill Source** and may differ from the source content when `agentcfg` needs to prepare the **Discovery Name**.
_Avoid_: Managed Skill Copy, Managed Skill Instance, Managed Source Tree, source tree, skill cache

**Source Skill Name**:
The name of a **Skill** as it appears in a **Skill Source**.
_Avoid_: original name, source name

**Discovery Name**:
The name under which a configured item is made discoverable to a **Client**. For V1 skills, the Discovery Name is either the Source Skill Name or the name chosen by a **Skill Alias**.
_Avoid_: discovered name, installed name, target name, alias target

**Discovery Name Collision**:
A conflict where two selected configured items would use the same **Discovery Name** at the same **Client Discovery Location**.
_Avoid_: alias collision, installed-name collision, target-name collision

**Skill Alias**:
A Config Layer rule that changes the **Discovery Name** for a selected **Skill**.
_Avoid_: alias, rename, rewrite

**Skill Selection**:
The resulting set of **Skills** chosen from a **Skill Source** by a **Config Layer**, whether chosen explicitly, through Skill Groups, or by selecting every discovered Skill in the source.
_Avoid_: selection, selected skills

**Included Skill**:
A **Skill** explicitly chosen by Source Skill Name from a **Skill Source**.
_Avoid_: include, include entry

**Skill Group**:
A source-defined named set of Source Skill Names that can be selected together.
_Avoid_: group, category, bundle

### Clients, Discovery, and Safety

**Client**:
An agent application or CLI that discovers configured capabilities from filesystem locations. V1 Clients include Codex, Pi, OpenCode, Claude Code, Cline, and Cursor.
_Avoid_: agent, app, consumer

**Client Discovery Location**:
A client-specific filesystem location that a **Client** scans to discover managed configuration. A Client Discovery Location is either project-level or user-level.
_Avoid_: Client Target, Client Install Location, Client Config Install Location, target, destination, install path

**Installed Artifact**:
A managed filesystem entry placed where a **Client** can discover it. For V1 skills, an Installed Artifact is one skill entry under a **Client Discovery Location**.
_Avoid_: target artifact, installed target, client target

**Unmanaged Artifact**:
A filesystem entry at a **Client Discovery Location** that is not recorded in the **Manifest** as owned by `agentcfg`.
_Avoid_: external artifact, existing artifact

**Managed State**:
`agentcfg`-owned state used to apply, inspect, and prune configuration safely. Managed State includes the Manifest and any managed content, such as Managed Skill Content.
_Avoid_: cache, generated state, internal state

**Client Discovery Registry**:
The built-in catalog of supported **Clients**, the configured item kinds they can discover, and the **Client Discovery Locations** used at each **Install Level**.
_Avoid_: client target registry, target registry, registry

**Discovery Requirement**:
A requirement from a **Config Layer** that a configured item be discoverable by a **Client** at an **Install Level**. An Installed Artifact may satisfy many Discovery Requirements.
_Avoid_: Consumer, Artifact Claim, Install Requirement, owner, reference

**Stale Discovery Requirement**:
A Discovery Requirement recorded in the **Manifest** that is no longer present in Desired State.
_Avoid_: stale consumer, stale claim, unmet requirement

**Unsatisfied Discovery Requirement**:
A Discovery Requirement in Desired State that does not currently have a valid Installed Artifact.
_Avoid_: stale requirement, missing install

**Stale Installed Artifact**:
An Installed Artifact recorded in the **Manifest** that has no remaining Discovery Requirements.
_Avoid_: stale target, stale artifact

**Unexpected Symlink Target**:
A symlink destination that differs from the destination recorded in the **Manifest** for an Installed Artifact.
_Avoid_: unexpected target, wrong target

**Broken Symlink**:
An Installed Artifact symlink whose destination does not exist.
_Avoid_: broken target, missing target

### Desired State and Resolution

**Desired State**:
The outcome active Config Layers ask `agentcfg` to make true, before source resolutions are fixed.
_Avoid_: wanted state, intended state

**Locked Desired State**:
Desired State after source resolutions are fixed so Apply can repeat the same result.
_Avoid_: locked state, resolved state

**Lockfile**:
A user-visible file that records Locked Desired State for Configured Items that need repeatable source resolution.
_Avoid_: lock, resolved state file

**Manifest**:
An `agentcfg`-owned state file that records Installed Artifacts and the Discovery Requirements that keep them present. The Manifest lets Status and Prune distinguish managed state from unmanaged files.
_Avoid_: ownership file, state file

**Source Refresh**:
The workflow option that refreshes source resolutions before producing Locked Desired State. In V1, Source Refresh refreshes Skill Source resolutions.
_Avoid_: upgrade, update

### Layers and Levels

**Config Layer**:
One user-authored configuration layer. V1 has three Config Layers: **Shared Project Config**, **User Project Config**, and **User Config**.
_Avoid_: config scope, layer scope

**Active Config Layers**:
The Config Layers selected for a workflow. Project-level workflows use Shared Project Config then User Project Config; user-level workflows use User Config.
_Avoid_: active scopes, selected layers

**Shared Project Config**:
A Config Layer for Agent Configuration intentionally shared by everyone working in one Project. At the Project Level, Shared Project Config is active alongside User Project Config.
_Avoid_: project config, shared scope

**User Project Config**:
A Config Layer for one User's additions in one Project. At the Project Level, User Project Config is active alongside Shared Project Config.
_Avoid_: local project config, personal project config, user project scope

**User Config**:
A Config Layer for one User's Agent Configuration across Projects. At the User Level, User Config is the only active Config Layer.
_Avoid_: global config, home config, user scope

**User**:
The current operating-system user whose user-level configuration and state `agentcfg` reads or writes.
_Avoid_: agent user, team member, account

**Project**:
A repository or working tree with a root where project-level Agent Configuration can live and Project Level workflows can apply.
_Avoid_: repo, workspace, directory

**Project Root**:
The root directory of a **Project** where project-level Config Layers, Lockfiles, Managed State, and Client Discovery Locations are resolved.
_Avoid_: repo root, workspace root

**Install Level**:
Whether `agentcfg` applies managed configuration at the project level or the user level. The Install Level determines which Config Layers are active and which Client Discovery Locations are considered.
_Avoid_: Install Scope, sync scope, target scope, command scope

**Project Level**:
The Install Level that applies project-scoped configuration to project-level Client Discovery Locations.
_Avoid_: project scope

**User Level**:
The Install Level that applies User Config to user-level Client Discovery Locations.
_Avoid_: user scope

**Persisted Scope Value**:
The string stored in a Config Layer to identify what kind of layer it is. V1 Persisted Scope Values are `shared-project`, `user-project`, and `user`.
_Avoid_: scope, TOML scope

### Workflows

**Preview**:
A read-only workflow that shows what **Apply** would change for the active configuration. Preview never writes config, lockfiles, manifests, managed state, source locations, or Client Discovery Locations.
_Avoid_: plan, dry run

**Apply**:
A one-way workflow that applies Locked Desired State from active configuration into managed state and Client Discovery Locations. Apply never writes changes back to source locations.
_Avoid_: sync, two-way sync, source update, bidirectional sync

**Prune**:
A workflow that removes stale managed state only when `agentcfg` can prove it is safe.
_Avoid_: clean, delete, uninstall

**Status**:
A workflow that reports whether the current managed install state is consistent for an Install Level.
_Avoid_: diagnose, inspect

**Doctor**:
A workflow that reports whether the local environment and configuration are capable of working.
_Avoid_: status, inspect

## Flagged Ambiguities

**Scope**:
Resolved as ambiguous. Use **Config Layer**, **Install Level**, or **Persisted Scope Value** instead.

**Source**:
Resolved as ambiguous. Use a resource-specific source term such as **Skill Source**; future source terms should be added only for configuration items that actually have an external origin to resolve.

**Source Location**:
Not a canonical term yet. Prefer resource-specific source terms such as **Skill Source** until multiple Configured Item kinds prove they share the same source-resolution lifecycle.

**Managed Source Tree**:
Resolved as ambiguous. Use **Managed Skill Content** for V1. Future configuration items that share the same source-derived managed-content lifecycle should follow the same pattern with resource-specific terms.

**Override**:
Avoid for V1 Project Level behavior. User Project Config is additive with Shared Project Config; do not say it overrides Shared Project Config.

**`--user`**:
Resolved as command-specific CLI syntax, not a domain term. For `init`, it selects **User Config**; for `preview`, `apply`, `prune`, and `status`, it selects the user **Install Level**.

**Sync**:
Resolved as misleading for the apply workflow because it suggests two-way synchronization. Use **Apply** for the one-way workflow from active configuration to managed state and Client Discovery Locations.

**Plan**:
Resolved as misleading for the read-only workflow. Use **Preview** for the workflow that shows what Apply would change without writing.

**Upgrade**:
Resolved as misleading for source resolution refresh. Use **Source Refresh** for the option that refreshes Skill Source resolutions before producing Locked Desired State; the CLI flag should be `--refresh-sources`.

## Example Dialogue

Dev: "Does V1 manage all agent configuration?"

Domain expert: "No. V1 manages Skill Configuration. Other Agent Configuration items may be added later."

Dev: "Is this command using user scope?"

Domain expert: "Say which one: `agentcfg init --user` selects the User Config Layer, while `agentcfg apply --user` selects the user Install Level."
