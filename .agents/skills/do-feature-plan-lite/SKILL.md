---
name: do-feature-plan-lite
description: Research and create a lightweight implementation plan for a feature, with compact design calibration when maintainability risk is present.
argument-hint: [feature-description]
---

# Planning: Feature Implementation Plan (Lite)

## Feature Description

$ARGUMENTS

## Mission

Create a compact implementation plan, usually 1-2 pages, that is executable without this conversation.

## Lite planning guardrails

- Keep the plan compact.
- Skip external research unless it is required to decide an approach.
- Focus on the minimum viable path.
- Avoid speculative future-proofing.
- Also avoid careless under-design when shared behavior, public contracts, or duplicated domain logic are involved.
- Limit the plan to 3-6 tasks with only necessary validation commands.
- Do not repeat the obvious. Reserve documentation/comment guidance for non-obvious behavior, edge cases, and invariants.

## Design discipline

Choose the smallest design that survives the next plausible change.
Avoid speculative architecture, but also avoid careless under-design.
Abstract shared meaning, not shared shape.
Abstractions must reduce caller complexity, clarify meaning, or protect invariants.
Push complexity into the module that can hide it best.

## Design risk tiers

| Tier | Meaning | Required behavior |
|---|---|---|
| Tier 0 | Mechanical or purely local | Implement directly; no visible design note unless a concern appears. |
| Tier 1 | Local but maintainability-relevant | Implement directly; run a lightweight maintainability check after code changes. |
| Tier 2 | Design-impacting but reversible | Inspect bounded context; use a compact design note or design lens; proceed unless asked to pause. |
| Tier 3 | Expensive to undo | Inspect broader context; use a full design pass; pause for approval before implementation or deviation. |

Tier from the request is a guess; tier after context scan is the decision.

## Quick process

1. Identify the closest existing feature or component.
2. Read relevant project guidance and nearby patterns.
3. Confirm scope and approach.
4. Assign design-risk tier.
5. Break work into 3-6 concrete tasks.
6. Include minimal validation commands.

## Compact design note

If the feature touches shared behavior, public contracts, module boundaries, validation/persistence/auth/config, or duplicated domain logic, include:

```markdown
Design note:
- Design risk: Tier 1 / Tier 2 / Tier 3
- Chosen shape:
- Why not simpler:
- Why not more abstract:
- Next plausible change considered:
```

For Tier 3, stop after the plan and mark approval needed before execution.

## Output

Save plan as:

```text
.agent-config/user/artifacts/plans/[3-digit-incrementing-number]-[feature-name].md
```

## Quality criteria

- [ ] Scope stays minimal and avoids speculative future work.
- [ ] Plan avoids careless under-design where shared meaning exists.
- [ ] Tasks are ordered and independently testable.
- [ ] Pattern references include file paths.
- [ ] Validation commands are executable and minimal.
- [ ] Acceptance criteria are measurable.

## Confirmation

After creating the plan, confirm:

- Feature name created.
- Plan saved to exact path.
- Design-risk tier assigned.
- Validation commands included.
- Another agent could execute this without context.
