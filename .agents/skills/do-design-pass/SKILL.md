---
name: do-design-pass
description: 'Use before implementing non-trivial changes that can shape future maintenance: public APIs, shared abstractions, module boundaries, persistence, validation, authorization, configuration, or behavior used by multiple callers.'
argument-hint: feature request, design question, or affected area
allowed-tools: Read, Bash(git:*)
---

# do-design-pass

## Purpose

Choose a maintainable design shape before implementation. This skill prevents both speculative architecture and under-designed tactical patches.

## Design discipline

Choose the smallest design that survives the next plausible change.
Avoid speculative architecture, but also avoid careless under-design.
Abstract shared meaning, not shared shape.
Abstractions must reduce caller complexity, clarify meaning, or protect invariants.
Push complexity into the module that can hide it best.
Treat awkward, leaky, flag-heavy, duplicated, or ceremonial implementation as design feedback.
No design vocabulary without code evidence and human-maintenance impact.

## Use when

Use this skill when the change is Tier 2 or Tier 3:

- It changes a public API, CLI, schema, event, contract, or data model.
- It changes module boundaries or ownership of behavior.
- It introduces or modifies a shared abstraction, helper, interface, factory, adapter, strategy, registry, or service layer.
- It affects persistence, configuration, validation, authorization, caching, retries, error handling, or cross-cutting policy.
- It affects behavior used by multiple callers or modules.
- It introduces flags, modes, options, or special cases.
- It duplicates logic that may represent the same domain concept, rule, or invariant.
- The direct fix would add coupling, branching, repetition, or scattered knowledge.
- The user asks for design first, plan only, architecture, refactoring strategy, or tradeoff analysis.

## Do not use when

Do not use this skill for:

- Formatting-only changes.
- Copy/text-only changes.
- Isolated tests that do not alter production design.
- Small local bug fixes with no shared behavior, new abstraction, or caller impact.
- Mechanical migrations where the design is already decided.
- Pure code review after implementation; use `do-design-review` instead.
- Final implementation sanity checks; use `do-maintainability-check` instead.

## Design risk tiers

| Tier | Meaning | Required behavior |
|---|---|---|
| Tier 0 | Mechanical or purely local | Implement directly; no visible design note. |
| Tier 1 | Local but maintainability-relevant | Implement directly; lightweight maintainability check after code changes. |
| Tier 2 | Design-impacting but reversible | Bounded context scan; compact design note; proceed unless asked to pause. |
| Tier 3 | Expensive to undo | Broader context scan; full design pass; pause for approval before implementation. |

Tier from the request is a guess; tier after context scan is the decision.

## Workflow

1. Read project guidance: `AGENTS.md`, repo instructions, local architecture notes.
2. Inspect directly affected files and nearby callers/callees.
3. Assign or revise the design-risk tier.
4. Compare at least two design shapes: direct/simple and more abstract.
5. Choose the smallest design that survives the next plausible change.
6. For Tier 2, explain and proceed unless the user asked to stop.
7. For Tier 3, explain and pause for approval.

## Output format

### Tier 2 — compact design note

```markdown
Design risk: Tier 2 — design-impacting but reversible.

Design note:
- Chosen shape:
- Why not simpler:
- Why not more abstract:
- Next plausible change considered:
- Complexity ownership:
- Proceeding because:
```

### Tier 3 — full design pass, pause for approval

```markdown
Design risk: Tier 3 — expensive to undo.

Design pass:
- Goal:
- Existing shape:
- Relevant local context:
- Option A — direct/simple:
- Option B — more abstract:
- Option C — alternate, if useful:
- Recommendation:
- Why not simpler:
- Why not more abstract:
- Complexity ownership:
- Public/API/schema/security impact:
- Reversibility:
- Decision needed:

Paused for approval before implementation.
```

## Constraints

- Show the reasoning needed to judge the decision; omit ceremony.
- Do not use design terms without code evidence and maintenance impact.
- Do not invent architecture from hypothetical scale.
- Do not ignore shared domain meaning just to keep the diff small.
- If project guidance is missing, default to the most reversible design and record the guidance gap.
