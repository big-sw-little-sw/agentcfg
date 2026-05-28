---
name: do-plan-adherence-review
description: Verify whether an implementation followed the approved plan and whether deviations were explicit, justified, approval-safe, and acceptance-preserving.
argument-hint: plan-file execution-report-or-diff
allowed-tools: Read, Bash(git:*)
---

# do-plan-adherence-review

## Purpose

Judge fidelity to the approved plan and deviation handling.

This skill does not primarily judge whether the plan was good. Use `do-design-review` to judge maintainability/design shape and `do-code-review` to judge correctness.

## Design risk tiers

| Tier | Meaning | Required behavior |
|---|---|---|
| Tier 0 | Mechanical or purely local | Implement directly; no visible design note unless a concern appears. |
| Tier 1 | Local but maintainability-relevant | Implement directly; run a lightweight maintainability check after code changes. |
| Tier 2 | Design-impacting but reversible | Inspect bounded context; use a compact design note or design lens; proceed unless asked to pause. |
| Tier 3 | Expensive to undo | Inspect broader context; use a full design pass; pause for approval before implementation or deviation. |

Tier from the request is a guess; tier after context scan is the decision.

## Required inputs

Use the best available form of:

- Original feature request.
- Approved plan.
- Final diff or implementation summary.
- Any `Plan Deviation Notice` sections.
- Acceptance criteria, if available.

If a required input is missing, state the limitation and review what is available.

## Review logic

1. Compare implementation against the original plan.
2. Identify completed, partially completed, missing, changed, and added work.
3. Read any `Plan Deviation Notice` sections.
4. Decide whether each deviation was:
   - explicit or silent
   - justified or unjustified
   - acceptance-preserving or acceptance-changing
   - reversible or expensive to undo
   - handled with the right approval behavior
5. Penalize silent deviations more heavily than explicit justified deviations.

## Verdicts

Adherent:

- Plan was followed materially.
- No meaningful silent deviations.

Adherent with justified deviations:

- Deviations were explicit, justified, reversible or approved, and did not alter acceptance criteria.

Non-adherent:

- Silent material deviations.
- Missing planned work.
- Added unplanned work with meaningful scope/design impact.
- Acceptance criteria changed without approval.
- Tier 3 deviation proceeded without approval.

## Output format

```markdown
Plan adherence review:
- Verdict: Adherent / Adherent with justified deviations / Non-adherent

Plan step coverage:
- Completed:
- Partially completed:
- Missing:
- Added outside plan:

Deviation handling:
- Explicit justified deviations:
- Questionable deviations:
- Silent deviations:

Acceptance criteria impact:
- None / possible / definite

Approval issues:
- Any Tier 3 deviation without approval?
- Any acceptance-changing deviation without approval?

Maintainability impact:
- Improved / unchanged / worsened

Verifier conclusion:
- ...
```

## Rules

- A justified deviation can pass verification; a silent deviation should not.
- Tier 3 or acceptance-changing deviations require approval.
- Do not mutate the plan while reviewing it.
- If the implementation is better than the plan but deviated silently, mark the design benefit separately from the adherence failure.
