---
name: do-design-review
description: 'Review an existing design, diff, PR, or implementation for long-term maintainability and human ownership risk: over/under-design, abstractions, duplication, interfaces, module boundaries, and complexity ownership.'
argument-hint: commit-hash, commit-range, PR/diff, or design description
allowed-tools: Read, Bash(git:*)
---

# do-design-review

## Purpose

Review whether humans will regret owning this design.

This is not a general bug review. For correctness, security bugs, test gaps, and edge cases, use the normal code-review workflow. Use this skill when the main question is maintainability and design shape.

## Design discipline

Choose the smallest design that survives the next plausible change.
Avoid speculative architecture, but also avoid careless under-design.
Abstract shared meaning, not shared shape.
Abstractions must reduce caller complexity, clarify meaning, or protect invariants.
Push complexity into the module that can hide it best.
Treat awkward, leaky, flag-heavy, duplicated, or ceremonial implementation as design feedback.
No design vocabulary without code evidence and human-maintenance impact.

## Design risk tiers

| Tier | Meaning | Required behavior |
|---|---|---|
| Tier 0 | Mechanical or purely local | Implement directly; no visible design note unless a concern appears. |
| Tier 1 | Local but maintainability-relevant | Implement directly; run a lightweight maintainability check after code changes. |
| Tier 2 | Design-impacting but reversible | Inspect bounded context; use a compact design note or design lens; proceed unless asked to pause. |
| Tier 3 | Expensive to undo | Inspect broader context; use a full design pass; pause for approval before implementation or deviation. |

Tier from the request is a guess; tier after context scan is the decision.

## Use when

Use this skill when reviewing:

- A completed implementation or diff.
- A PR/MR where maintainability/design quality is in question.
- A proposed abstraction, API, module boundary, or refactor.
- Code that feels verbose, over-engineered, under-designed, duplicated, or hard to maintain.
- Changes involving shared behavior, public contracts, validation, persistence, authorization, configuration, retries, caching, or error handling.
- Agent-generated code that may be correct but does not feel like code a human would choose to maintain.

If no explicit target is provided, inspect the current working tree diff.

## Do not use when

Do not use this skill when:

- The user only wants bug finding, security review, or test coverage review.
- The diff is purely mechanical, formatting-only, or copy-only.
- The review would be style nitpicking without future maintenance impact.
- The code is intentionally temporary and clearly isolated.
- The user asks for implementation, not review; use `do-design-pass` first if design risk is present.

## Scope modes

### Default review

- Review the provided target or current diff.
- Expand context only around design risk.
- Return the top few findings.

### Full review

Use when the user explicitly asks for a full/thorough review.

- Inspect the full target, affected modules, relevant callers/callees, tests, public contracts, and project guidance.
- Consider each design-risk category.
- Parallelize by review lens if the environment supports it.
- If subagents are unavailable, use the same lenses sequentially.
- Synthesize findings into one prioritized review.

Review lenses for full review:

1. APIs, contracts, schemas, module boundaries.
2. Abstractions, duplication, over/under-design.
3. Complexity ownership, information leakage, caller burden.
4. Flags/options/special cases, comments, tests as design signals.

## Severity model

Must fix:

- Likely to cause repeated bugs, broad change amplification, leaky coupling, or hard-to-reverse API damage.

Should fix:

- Creates avoidable complexity, unclear ownership, or duplication that will probably spread.

Watch:

- Slight smell, but acceptable given scope, reversibility, or weak evidence.

## Verdict logic

Accept:

- No material design risks found. Any concerns are minor, reversible, or local.

Accept with concerns:

- Design is workable, but has one or more localized/reversible issues that may create future maintenance pain.

Redesign recommended:

- The current shape is likely to cause broad change amplification, leaky coupling, hard-to-reverse API damage, duplicated domain policy, misplaced complexity, or significant over/under-design.

Caps:

- Any Must fix issue prevents an Accept verdict.
- An unresolved Tier 3 design issue usually means Redesign recommended.
- Multiple Should fix issues across the same design area may become Redesign recommended.
- Watch issues alone should not block Accept.

## Output format

```markdown
Design review result:
- Scope: Default / Full
- Verdict: Accept / Accept with concerns / Redesign recommended
- Design risk tier:
- Files/areas reviewed:
- Top findings:
  1. ...
  2. ...
  3. ...
- Smallest strategic fix:
- What not to change:
- Review limitations:
```

For full review mode, add:

```markdown
Lens coverage:
- APIs/contracts/module boundaries:
- Abstractions/duplication/over-under-design:
- Complexity ownership/information leakage:
- Flags/options/special cases/comments:
- Tests as design signals:
```

For each issue:

```markdown
Issue:
- Severity: Must fix / Should fix / Watch
- Evidence:
- Future maintenance impact:
- Smallest strategic fix:
```

## Constraints

- Report only design issues that predict future human maintenance pain.
- Prioritize the top few findings.
- Do not reward cleverness, ceremony, or extra abstraction.
- Do not nitpick style unless it affects maintainability.
- Always include code evidence and maintenance impact.
- Include what not to change to prevent over-refactoring.
