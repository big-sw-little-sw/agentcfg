---
name: do-execution-report
description: Generate implementation report for system review, preserving plan-adherence evidence, plan deviations, validation results, and maintainability evidence.
---

# Execution Report

## Design risk tiers

| Tier | Meaning | Required behavior |
|---|---|---|
| Tier 0 | Mechanical or purely local | Implement directly; no visible design note unless a concern appears. |
| Tier 1 | Local but maintainability-relevant | Implement directly; run a lightweight maintainability check after code changes. |
| Tier 2 | Design-impacting but reversible | Inspect bounded context; use a compact design note or design lens; proceed unless asked to pause. |
| Tier 3 | Expensive to undo | Inspect broader context; use a full design pass; pause for approval before implementation or deviation. |

Tier from the request is a guess; tier after context scan is the decision.

Review and analyze the implementation you just completed.

Save to:

```text
.agent-config/user/artifacts/execution-reports/[feature-name].md
```

## Report structure

```markdown
# Execution Report: [feature-name]

## Meta Information
- Plan file:
- Feature/request:
- Date:
- Files added:
- Files modified:
- Lines changed:

## Plan Adherence
- Original plan followed: Yes / Mostly / No
- Planned steps completed:
- Planned steps changed:
- Planned steps skipped:
- Unplanned work added:

## Plan Deviations
- None.
```

When deviations exist:

```markdown
## Plan Deviations

1. <short title>
   - Plan step affected:
   - Deviation type:
   - Design risk tier:
   - Acceptance criteria impact:
   - Maintainability impact:
   - Approval status:
   - Verifier note:
```

Continue with:

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
- Human-maintainer concern:
- Follow-up needed:

## Validation Results
- Syntax & linting:
- Type checking:
- Unit tests:
- Integration tests:
- Other validation:

## What Went Well
- ...

## Challenges Encountered
- ...

## Skipped Items
- ...

## Recommendations
- Project guidance updates:
- Plan skill updates:
- Execution skill updates:
- Calibration examples to add:
```

## Boundaries

- Preserve evidence; do not deeply judge it here.
- `do-plan-adherence-review` judges plan fidelity.
- `do-design-review` judges maintainability/design shape.
- `do-system-review` decides whether workflow/guidance changes are needed.
