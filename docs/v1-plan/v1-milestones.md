# V1 Milestones Plan

This document records the high-level implementation sequence for V1. It stops at milestone boundaries; task breakdowns will be added later.

Authoritative product and design inputs:

- [CONTEXT.md](../../CONTEXT.md) for user-facing terms and API language.
- [docs/prd.md](../prd.md) for command behavior and acceptance criteria.
- [docs/design-v1.md](../design-v1.md) for module boundaries, state shapes, safety rules, and command flows.

## Sequencing Principle

Implement V1 from stable contracts and pure leaves upward, but validate through command-shaped vertical slices early. Pure modules should not be completed so far ahead of workflow wiring that their shape stops being tested against `preview`, `apply`, `prune`, and `status`.

The intended order is:

```text
contracts -> leaf modules -> planning and policy -> execution -> workflows and CLI
```

V1 is not just a filesystem installer. It is a safety-oriented reconciler system where source resolution, current-state evidence, command policy, and mutation ordering each fail differently. The implementation should make those boundaries visible early enough that tests can pin behavior before later milestones depend on it.

## Design Note

- Design risk: Tier 2.
- Chosen shape: five coarse milestones aligned to the V1 design's module boundaries.
- Why not simpler: a single MVP milestone would blur contracts, policy, mutation safety, and CLI wiring, making review and recovery harder.
- Why not more abstract: V1 manages Skills only; the plan should not introduce a generic resource graph or platform roadmap.
- Next plausible change considered: later Configured Item kinds can be added above or beside Skills if the V1 contracts stay typed and item-specific.

## Milestones

### M1: Stable Contracts And Workspace Boundary

Establish the Rust workspace shape, shared result/error conventions, command request vocabulary, config and lockfile model boundaries, Manifest model, and typed state aggregates for `DesiredState`, `LockedDesiredState`, `ProposedLockedDesiredState`, and `CurrentState`.

Importance: contracts are load-bearing. Vague shapes will leak CLI wording, filesystem details, or persistence schema into modules that should exchange normalized state. This milestone is complete when data passed between modules has stable names, the CLI/core boundary is thin, and the codebase compiles and tests without real install behavior.

Task plans: [M1 Stable Contracts And Workspace Boundary](m1-stable-contracts-and-workspace-boundary/README.md).

### M2: Focused Leaf Modules

Implement the low-level modules that return facts or perform narrow persistence operations without command policy: client discovery registry, filesystem probing, deterministic content digesting, config and lock stores, Manifest store, Managed Skill Content store, and discovery symlink operations.

Importance: leaf modules are where ad hoc parsing and duplicated probing most easily creep in. `content_digest` especially needs deterministic tests because it affects lock planning, content-addressed Managed Skill Content, artifact updates, and interrupted-write recovery. This milestone is complete when higher modules can rely on structured facts and narrow persistence APIs instead of re-probing filesystem state.

### M3: Planning, Inventory, And Reconciliation Policy

Implement the pure planning and policy modules: `desired_builder`, `lock_planner`, `current_inventory`, and `reconciler`. The reconciler owns command-specific lifecycle policy for preview, apply, prune, and status while remaining independent of source resolution, lockfile persistence, and filesystem mutation.

Importance: this is where config intent, repeatable locking, observed evidence, and lifecycle policy become separate. The reconciler should be one of the most heavily tested modules in V1 because bugs in blockers, stale state, prune skips, status findings, or unmanaged artifact refusal become user-visible safety defects. This milestone is complete when locked desired state and current state produce structured `PreviewReport`, `ApplyPlan`, `PrunePlan`, and `StatusReport` values with tested edge cases.

### M4: Executor And Safe Mutation

Implement apply and prune execution against approved plans. The executor owns private preflight, write ordering, last-mile filesystem safety, Managed Skill Content materialization, discovery symlink creation or update, lockfile writes, and Manifest updates.

Importance: the executor is where policy becomes filesystem state, so it must revalidate before mutation. Apply ordering is a safety invariant: lockfiles, Managed Skill Content, discovery artifacts, then Manifest ownership. This milestone is complete when apply refuses blockers before mutation, records ownership last, supports forward recovery, and prune removes only manifest-owned stale state while skipping unsafe removals.

### M5: Workflows, CLI, Reporting, And End-To-End Acceptance

Wire command use cases, CLI parsing, reporter output, and exit-code mapping around the core modules. `preview`, `apply`, `prune`, `status`, and `doctor` should follow the command flows in `docs/design-v1.md`, with `init` creating the selected Config Layer.

Importance: workflows preserve command sequencing so the CLI stays thin, and reporting turns structured results into precise user-facing diagnostics. End-to-end tests should prove behavior isolated module tests cannot: read-only preview, repeatable apply, explicit prune, shared Client Discovery Locations, User Level separation, client filters, and Source Refresh. This milestone is complete when the MVP acceptance criteria in `docs/prd.md` pass end to end.

## Validation Baseline

Milestone-specific validation will be defined during task breakdown. The default validation baseline is:

```bash
scripts/validate-all.sh
```

This uses the repo-wide validation policy from [AGENTS.md](../../AGENTS.md): file hygiene, Rust formatting, and linting run through `prek`, while only the branch-protection hook is skipped for local validation. Tests run separately through Cargo inside the script.
