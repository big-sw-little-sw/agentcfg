---
name: do-issue-fix
description: Fix a described bug or issue while keeping changes focused and escalating design-impacting fixes through the design-pass workflow.
argument-hint: description of the problem to fix
allowed-tools: Read, Write, Edit, Grep, Bash(uv run ruff:*), Bash(uv run pyright:*), Bash(uv run pytest:*), Bash(git:*)
---

# Fix Issue: Address a Described Problem

## Problem Description

**User's description**: $ARGUMENTS

## Design discipline

Choose the smallest design that survives the next plausible change.
Avoid speculative architecture, but also avoid careless under-design.
Abstract shared meaning, not shared shape.
Abstractions must reduce caller complexity, clarify meaning, or protect invariants.
Push complexity into the module that can hide it best.
Treat awkward, leaky, flag-heavy, duplicated, or ceremonial implementation as design feedback.

## Phase 1: Understand the problem

Parse the description:

- What is broken or needs fixing?
- Where might it live?
- What severity is implied?
- What acceptance behavior must remain unchanged?

Search systematically:

- Prefer LSP for symbols/references when available.
- Use Grep as fallback.
- Read candidate files before changing anything.
- Do not guess locations or make random changes.

## Phase 2: Classify design risk

Before implementing, classify the fix:

## Design risk tiers

- Tier 0/1: local, reversible, no public/shared behavior impact.
- Tier 2: design-impacting but reversible; shared helper, duplicated domain rule, internal abstraction, behavior used by multiple callers.
- Tier 3: expensive to undo; public API, schema/persistence, auth/security, cross-cutting policy, major module boundary.

For Tier 2/3, run or apply `do-design-pass` behavior before editing.

Do not sneak a redesign into a bug fix.

## Phase 3: Create fix plan

Explain briefly:

- Issue found in:
- What is wrong:
- Why it matters:
- Proposed fix:
- Design risk tier:
- Whether approval is needed:

Wait for confirmation if the issue is unclear, high-risk, or Tier 3. Proceed directly only when the fix is obvious and low-risk.

## Phase 4: Implement the fix

- Follow `AGENTS.md` standards.
- Maintain existing code style and patterns.
- Keep changes minimal and focused.
- Add or update tests when behavior changes or the bug should be prevented from recurring.
- If the direct fix creates duplication, flags, leaky interfaces, or scattered knowledge, treat that as design feedback.

## Phase 5: Validate

Run the relevant validation commands from `AGENTS.md`, project config, package scripts, Makefiles, tox/nox config, or nearby CI config.

If validation fails:

- Fix the regression.
- Re-run validation.
- Continue only when validation passes or clearly report the blocker.

## Phase 6: Maintainability check

After non-trivial fixes, run:

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

## Summary report

```markdown
### Fix Applied

Problem described:
- ...

Issue found in:
- `path/to/file.py:123`

Severity:
- critical / high / medium / low

Design risk tier:
- Tier 0 / Tier 1 / Tier 2 / Tier 3

What was fixed:
- ...

Files modified:
- ...

Tests added/updated:
- ...

Validation results:
- ...

Maintainability check:
- ...

Ready for commit:
- Yes / No
```
