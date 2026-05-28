---
name: do-plan-execution
description: Execute an implementation plan while preserving plan adherence, recording explicit deviations, and finishing with maintainability evidence.
argument-hint: [path-to-plan]
allowed-tools: Read, Write, Edit, Bash(uv:*), Bash(ruff:*), Bash(pyright:*), Bash(pytest:*), Bash(npm:*), Bash(bun:*), Bash(git:*)
---

# Execute: Implement from Plan

## Plan to execute

Read plan file: `$ARGUMENTS`

## Core rule

Do not mutate the plan. Record deviations against it.

A plan is the verifier's source of truth. If implementation reality conflicts with the plan, emit a structured `Plan Deviation Notice` instead of silently drifting or rewriting the plan.

## Design discipline

Choose the smallest design that survives the next plausible change.
Avoid speculative architecture, but also avoid careless under-design.
Abstract shared meaning, not shared shape.
Abstractions must reduce caller complexity, clarify meaning, or protect invariants.
Push complexity into the module that can hide it best.
Treat awkward, leaky, flag-heavy, duplicated, or ceremonial implementation as design feedback.

## Design risk tiers

| Tier | Meaning | Required behavior |
|---|---|---|
| Tier 0 | Mechanical or purely local | Implement directly; no visible design note unless a concern appears. |
| Tier 1 | Local but maintainability-relevant | Implement directly; run a lightweight maintainability check after code changes. |
| Tier 2 | Design-impacting but reversible | Inspect bounded context; use a compact design note or design lens; proceed unless asked to pause. |
| Tier 3 | Expensive to undo | Inspect broader context; use a full design pass; pause for approval before implementation or deviation. |

Tier from the request is a guess; tier after context scan is the decision.

## Execution instructions

### 1. Read and understand

- Read the entire plan.
- Read all referenced files and documentation needed to validate that the plan is sound.
- Review `AGENTS.md` and validation commands.
- Identify Tier 3 or approval-needed decisions before editing.

### 2. Execute tasks in order

For each planned task:

- Identify the file and action required.
- Read existing related files before modifying.
- Follow the plan and project guidance.
- Keep changes focused on the plan.

### 3. Handle implementation friction

If implementation reveals that the plan is awkward, leaky, over-engineered, under-designed, flag-heavy, duplication-prone, contradicted by code/tests, or requires scattered special cases, emit:

```markdown
## Plan Deviation Notice

Plan step affected:
- <quote or reference the original plan step>

Deviation type:
- Design friction / missing context / implementation constraint / validation failure / user-facing behavior risk / other

What implementation revealed:
- ...

Why the original plan step is problematic:
- ...

Design risk tier:
- Tier 1 / Tier 2 / Tier 3

Proposed action:
- Continue with deviation / pause for approval / stop and request plan revision

Impact on acceptance criteria:
- None / possible impact / definite impact

Impact on maintainability:
- ...

Verifier note:
- This implementation intentionally deviates from the original plan in the following way: ...
```

Behavior:

- Tier 1: emit notice and continue if acceptance criteria are unchanged.
- Tier 2: emit notice prominently; continue only if reversible and acceptance criteria are unchanged.
- Tier 3: emit notice and pause for approval before proceeding.
- Any acceptance-criteria change: pause. Do not continue silently.

### 4. Implement testing strategy

- Create test TODOs before implementation when useful.
- Implement tests specified in the plan.
- Add tests for changed behavior, especially bugs and invariants.
- Follow the repo's test organization.

### 5. Run validation commands

Run all validation commands from the plan in order.

If any command fails:

- Fix the issue.
- Re-run the command.
- Continue only when it passes or clearly report inability to pass.

### 6. Maintainability check

After non-trivial implementation, run the maintainability check:

```markdown
## Maintainability Check
- Design risk tier:
- Complexity added:
- Complexity removed/localized:
- Abstractions justified:
- Duplication:
- Flags/options/special cases:
- Caller burden:
- Knowledge leakage:
- Comments:
- Human-maintainer concern:
- Follow-up needed:
```

### 7. Final output report

Provide:

```markdown
## Completed Tasks
- ...

## Files Changed
- Created:
- Modified:

## Tests Added/Updated
- ...

## Validation Results
- ...

## Plan Deviations
- None.
```

or, when deviations exist:

```markdown
## Plan Deviations

1. <short title>
   - Plan step:
   - Decision:
   - Acceptance impact:
   - Maintainer impact:
   - Approval status:
   - Verifier note:
```

Then include the Maintainability Check.

## Important reminders

- Do not revise the plan file unless the user explicitly asks.
- Do not silently add unplanned scope.
- Do not silently change acceptance criteria.
- If the plan is wrong, record the deviation and follow the tier behavior.
- If a design-related deviation is Tier 3, pause before implementation.
