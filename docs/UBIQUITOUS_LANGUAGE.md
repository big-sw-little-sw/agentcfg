# Agent Configuration Management

Agent Configuration Management defines how agent-facing configuration becomes a repeatable, managed installation. The current implementation scope is Skill Configuration. Future configured item kinds should add their own configuration vocabulary without changing the shared installation and ownership language.

`agentcfg` derives ownership from lockfiles, deterministic managed paths, Client Search Locations, binding artifact shape, and symlink targets.

## Language

### Agent Configuration and Configured Items

**Agent Configuration**:
User-authored intent for which **Configured Items** should be available. The current implementation supports **Skill Configuration** only.
_Avoid_: agent config, settings

**Skill Configuration**:
The currently implemented part of **Agent Configuration** that declares which skills should be available to which **Clients**.
_Avoid_: skill setup, skill install config

**Configured Item**:
One kind of agent-facing thing managed by `agentcfg`. The current configured item kind is **Skill**. Future configured item kinds belong here with their own configuration vocabulary.
_Avoid_: resource, object, config item

**Agent Skill Format**:
The `SKILL.md` directory format used by the Agent Skills ecosystem.
_Avoid_: Agent Skills Standard, standard SKILL.md format

**Skill**:
A **Configured Item** in the **Agent Skill Format** that a **Client** can load and use.
_Avoid_: plugin, capability, tool

**Skill Source**:
A filesystem path or git location that contains one or more **Skills** available for selection.
_Avoid_: Source, repository, source tree

**Source Enumeration**:
The item-kind-specific process that lists available configured items from a source before selection and **Source Resolution**. Current skill support enumerates Source Skill Names and Skill Groups from a Skill Source.
_Avoid_: source discovery, discover skills

**Source Resolution**:
The item-kind-specific process that fixes source references, content identities, and other repeatability inputs before producing a **PinnedConfig**. Current skill support resolves Skill Source refs and skill content identities.
_Avoid_: source discovery, source lookup

**Managed Skill Tree**:
An `agentcfg`-owned skill directory prepared from a **PinnedConfig** and stored in **Managed State**. A Managed Skill Tree is derived from a **Skill Source** and may differ from the Skill Source content when `agentcfg` needs item-wide preparation for the **Configured Item Name**.
_Avoid_: Managed Skill Content, Managed Skill Copy, Managed Skill Instance, Managed Source Tree, source tree, skill cache

**Source Skill Name**:
The name of a **Skill** as it appears in a **Skill Source**.
_Avoid_: original name, source name

**Configured Item Name**:
The item-wide name after configuration is normalized. For current skill support, the Configured Item Name is either the Source Skill Name or the name chosen by a **Skill Alias**. Client bindings use the Configured Item Name unless a client-specific adapter requires otherwise.
_Avoid_: Discovery Name, discovered name, installed name, target name, alias target

**Skill Alias**:
A Config Layer rule that changes the **Configured Item Name** for a selected **Skill**. A Skill Alias is item-wide configuration, not **Client Adaptation**.
_Avoid_: alias, rename, rewrite

**Skill Selection**:
The resulting set of **Skills** chosen from a **Skill Source** by a **Config Layer**, whether chosen explicitly, through Skill Groups, or by selecting every enumerated Skill in that Skill Source.
_Avoid_: selection, selected skills

**Included Skill**:
A **Skill** explicitly chosen by Source Skill Name from a **Skill Source**.
_Avoid_: include, include entry

**Skill Group**:
A named set of Source Skill Names defined by a Skill Source that can be selected together.
_Avoid_: group, category, bundle

### Clients, Search, and Safety

**Client**:
An agent application or CLI that loads **Configured Items** from **Client Search Locations**. Supported Clients currently include Codex, Pi, OpenCode, Claude Code, Cline, and Cursor.
_Avoid_: agent, app, consumer

**Client Search Location**:
A client-specific filesystem location that a **Client** scans or loads from. A Client Search Location is either project-level or user-level.
_Avoid_: Client Target, Client Install Location, Client Config Install Location, target, destination, install path

**Managed State**:
`agentcfg`-owned filesystem state used to apply, inspect, and prune configuration safely. Managed State stores managed item-kind artifacts; current skill support stores Managed Skill Trees.
_Avoid_: cache, generated state, internal state

**Client Adapter Catalog**:
The built-in catalog of supported **Clients**, the configured item kinds they can load, their **Client Search Locations**, and the artifact shapes their adapters use at each **Install Level**.
_Avoid_: Client Discovery Registry, client target registry, target registry, registry

**Client Binding**:
The planned relationship that makes one configured item available to one **Client** at one **Install Level**, usually under its **Configured Item Name** and at one **Client Search Location**. A Client Binding is derived during configuration resolution and may be recorded as part of a **PinnedConfig**, but it is not a separate ownership record.
_Avoid_: Client Exposure, Consumer, Artifact Claim, Install Requirement, owner, reference

**Client Binding Artifact**:
A filesystem entry at a **Client Search Location** that realizes a **Client Binding**. The artifact shape is determined by the configured item kind and Client. Current skill support may realize a binding with a symlink under the **Configured Item Name**.
_Avoid_: Installed Artifact, target artifact, installed target, client target

**Client Adaptation**:
A client-specific content or layout change needed before a **Client** can load a configured item, such as frontmatter rewriting or client-specific file layout. Client Adaptation is not used for item-wide configuration changes such as **Skill Aliases**.
_Avoid_: alias, rename, global adaptation

**Managed Artifact**:
A filesystem entry that `agentcfg` may create, update, or remove because **Derived Ownership** proves it belongs to the managed installation. Current skill Managed Artifacts include Managed Skill Trees and Client Binding Artifacts.
_Avoid_: Installed Artifact, managed file, managed target

**Derived Ownership**:
The safety rule that `agentcfg` treats a filesystem entry as managed only when ownership can be proven from the **LockfilePinnedConfig**, the **Client Adapter Catalog**, deterministic Managed State paths, Client Binding Artifact shape, or symlink targets. Current artifacts are usually proven from LockfilePinnedConfig; stale artifacts must still be proven from managed path shape, binding artifact shape, or symlink target. If ownership cannot be derived, `agentcfg` must treat the entry as an **Unmanaged Conflict** instead of mutating it.
_Avoid_: ownership claim, owner marker

**Unmanaged Conflict**:
A filesystem entry at a path `agentcfg` needs to create, update, or remove, but whose ownership cannot be derived. An Unmanaged Conflict blocks automatic mutation: `agentcfg` must report it instead of overwriting or deleting it. Examples include a non-symlink at a required client binding path, a symlink to a non-managed target, or an entry with an unexpected shape inside a managed path.
_Avoid_: Unmanaged Artifact, external artifact, existing artifact

**Stale Managed Artifact**:
A **Managed Artifact** that still exists but is no longer required by the current **LockfilePinnedConfig** for the active **Install Level**.
_Avoid_: stale installed artifact, stale target, stale artifact

**Conservative Prune**:
The pruning policy that removes only **Stale Managed Artifacts** whose ownership can still be derived. Conservative Prune never removes an **Unmanaged Conflict**.
_Avoid_: clean, force prune, uninstall

### Configuration Resolution and Installation

**ConfigDoc**:
The parsed persisted configuration schema for one **Config Layer**. A ConfigDoc is not yet normalized for command options or item-kind-specific source resolution.
_Avoid_: config model, parsed config

**ConfigRequest**:
The normalized command-time request built from active **ConfigDocs** and command options, including Install Level and client selection. A ConfigRequest is not repeatable until item-kind-specific source references, selections, aliases, Configured Item Names, Client Bindings, and content identities are fixed.
_Avoid_: Desired State, wanted state, active config intent

**PinnedConfig**:
A **ConfigRequest** after **Source Resolution**, selections, aliases, **Configured Item Names**, **Client Bindings**, and content identities are fixed. For current skill support, this includes Skill Source refs, Skill Selections, and Skill Aliases. PinnedConfig is the repeatable configuration used for installation and consistency checks.
_Avoid_: Desired State, Locked Desired State, resolved state

**LockfilePinnedConfig**:
A **PinnedConfig** loaded from current **Lockfiles**.
_Avoid_: Locked Desired State, locked state

**PlannedPinnedConfig**:
A **PinnedConfig** proposed by Preview or Install resolution. Preview keeps it in memory; Install persists corresponding lockfile changes before installation.
_Avoid_: proposed locked state, preview plan

**ObservedInstallation**:
Observed install reality from the filesystem, **Managed State**, and **Client Search Locations**. ObservedInstallation classifies expected Managed Artifacts, Stale Managed Artifacts, missing Client Binding Artifacts, broken binding symlinks, unexpected binding targets, and Unmanaged Conflicts. ObservedInstallation may drift from **LockfilePinnedConfig** when files are changed manually, become broken, or have not been pruned.
_Avoid_: Current State, current state

**InstallPlan**:
The concrete installation mutation plan derived by comparing a **PlannedPinnedConfig** with an **ObservedInstallation**.
_Avoid_: apply plan, apply script, filesystem script

**InstallResult**:
The outcome of executing an **InstallPlan**, including completed changes, blockers, failures, and recovery diagnostics.
_Avoid_: result log

**Lockfile**:
A user-visible file that records **PinnedConfig** for Configured Items that need repeatable source resolution. Current skill lockfiles record repeatable Skill Source resolution.
_Avoid_: lock, resolved state file

**Source Refresh**:
The workflow option that reruns **Source Resolution** before producing a **PlannedPinnedConfig**. Current skill support refreshes Source Resolution for Skill Sources.
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
A repository or working tree with a root where project-level Agent Configuration can live and Project Level workflows can run.
_Avoid_: repo, workspace, directory

**Project Root**:
The root directory of a **Project** where project-level Config Layers, Lockfiles, Managed State, and Client Search Locations are resolved.
_Avoid_: repo root, workspace root

**Install Level**:
Whether `agentcfg` applies managed configuration at the **Project Level** or the **User Level**. The Install Level determines which Config Layers are active and which Client Search Locations are considered.
_Avoid_: Install Scope, sync scope, target scope, command scope

**Project Level**:
The Install Level that applies Project Level configuration to project-level Client Search Locations.
_Avoid_: project scope

**User Level**:
The Install Level that applies User Config to user-level Client Search Locations.
_Avoid_: user scope

**Persisted Config Layer Value**:
The string stored in the persisted `config-layer` field to identify what kind of Config Layer a file is. V1 Persisted Config Layer Values are `shared-project`, `user-project`, and `user`.
_Avoid_: persisted scope value, scope, TOML scope

### Workflows

**Add**:
A workflow that adds a configured item to a **Config Layer** and then may run **Install** to materialize the updated configuration.
_Avoid_: add install, include

**Remove**:
A workflow that removes a configured item from a **Config Layer** and then may run **Install** or **Prune** to update managed artifacts.
_Avoid_: uninstall, delete

**Preview**:
A read-only workflow that shows what **Install** would change for the active **ConfigRequest**. Preview never writes config, lockfiles, Managed State, item sources, or Client Search Locations.
_Avoid_: plan, dry run

**Install**:
A one-way workflow that resolves active configuration to a **PlannedPinnedConfig**, persists required lockfile changes, and installs Managed Artifacts. Current skill Install installs Managed Skill Trees and Client Binding Artifacts. Install never writes changes back to item sources.
_Avoid_: apply, sync, two-way sync, source update, bidirectional sync

**Prune**:
A workflow that applies **Conservative Prune** to remove Stale Managed Artifacts only when `agentcfg` can prove it is safe.
_Avoid_: clean, delete, uninstall

**Status**:
A workflow that reports whether **ObservedInstallation** is consistent with the **LockfilePinnedConfig** for an Install Level.
_Avoid_: diagnose, inspect

**Doctor**:
A workflow that reports whether the local environment and configuration are capable of working.
_Avoid_: status, inspect
