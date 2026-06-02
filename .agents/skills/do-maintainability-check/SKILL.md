---
name: do-maintainability-check
description: Run a compact final check after non-trivial implementation to verify the code remains human-maintainable without unnecessary abstraction, sloppy duplication, leaky interfaces, or misplaced complexity.
argument-hint: changed files, commit-hash, or summary of implementation
allowed-tools: Read, Bash(git:*)
---

# do-maintainability-check

## Purpose

Check whether the final code is easier for humans to own, not merely different or complete.

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

Use this skill after implementation when:

- The change was Tier 1, Tier 2, or Tier 3.
- The implementation introduced or removed an abstraction.
- The implementation added a helper, shared function, public method, mode, option, or special case.
- The implementation touched code used by multiple callers.
- The implementation duplicated logic or consolidated duplicated logic.
- The implementation changed comments around non-obvious behavior.
- The implementation changed validation, error handling, persistence, configuration, authorization, retries, or caching.
- The design changed during implementation.

If no explicit target is provided, inspect the current working tree diff.

## Do not use when

Do not use this skill when:

- No code was changed.
- The task was only to produce a design or plan.
- The task was only to review an existing diff; use `do-design-review`.
- The edit was purely mechanical and local, with no maintainability implications.
- The user explicitly asks to skip final review/checks.

## Output format

```markdown
Maintainability check:
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

## Behavior

- Keep it short. One line per item is usually enough.
- If clean, say so briefly.
- If a concern is minor and fixable, fix it before finalizing when the surrounding task already includes implementation.
- If a concern remains, explain why it is acceptable for now.
- If a concern is serious, recommend design review or revision before claiming done.

## Constraints

- Do not turn this into a full design review.
- Do not reward cleverness, ceremony, or extra abstraction.
- Do not report style preferences unless they predict future maintenance pain.
