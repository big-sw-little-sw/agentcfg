---
name: do-feature-plan
description: Research and create an implementation plan for a feature, including calibrated design-risk assessment and maintainability decisions.
argument-hint: [feature-description]
---

# Planning: Feature Implementation Plan

## Feature Description

$ARGUMENTS

## Mission

Transform a feature request into a context-rich implementation plan that another agent can execute without this conversation.

We do **not** write code in this phase.

The plan must choose the design shape, not merely list implementation steps.

## Design discipline

Choose the smallest design that survives the next plausible change.
Avoid speculative architecture, but also avoid careless under-design.
Abstract shared meaning, not shared shape.
Abstractions must reduce caller complexity, clarify meaning, or protect invariants.
Push complexity into the module that can hide it best.
Treat awkward, leaky, flag-heavy, duplicated, or ceremonial implementation as design feedback.
No design vocabulary without code evidence and human-maintenance impact.

## Determine Feature Name

Create a concise kebab-case feature name.

Plan path:

```text
.agent-config/user/artifacts/plans/[3-digit-incrementing-number]-[feature-name].md
```

## Planning Process

### Phase 1: Feature understanding

- Extract the core problem being solved.
- Identify user/system value.
- Determine feature type: new capability, enhancement, refactor, or bug fix.
- Map affected systems and components.
- Identify acceptance criteria.
- Ask for clarification before continuing if requirements or acceptance criteria are unclear.

### Phase 2: Codebase intelligence

Read project guidance first:

- `AGENTS.md`
- `CLAUDE.md` or similar repo instructions, if present
- local architecture docs, if present

Inspect bounded context:

- affected files
- nearby callers/callees
- existing patterns for similar behavior
- validation, authorization, persistence, error handling, configuration, and testing patterns relevant to the feature
- public contracts that may be expensive to change

### Phase 3: Design calibration

Assign a design-risk tier after context scan.

## Design risk tiers

| Tier | Meaning | Planning behavior |
|---|---|---|
| Tier 0 | Mechanical or purely local | Keep plan minimal. |
| Tier 1 | Local but maintainability-relevant | Include enough context to avoid sloppy local design. |
| Tier 2 | Design-impacting but reversible | Include Design Calibration section. |
| Tier 3 | Expensive to undo | Include Design Calibration section and mark approval-needed decisions. |

For Tier 2/3 work, include:

```markdown
## Design Calibration

Design risk:
- Tier 2 / Tier 3
- Why:

Existing shape:
- Relevant modules/patterns/contracts:

Option A — direct/simple:
- ...

Option B — more abstract:
- ...

Chosen shape:
- ...

Why not simpler:
- ...

Why not more abstract:
- ...

Next plausible change considered:
- ...

Complexity ownership:
- Where validation/policy/state/error handling/etc. should live:

Approval needed:
- Yes/No
- Reason:
```

For Tier 3, clearly mark what must be approved before execution.

### Phase 4: External research, only when needed

Use external research only when it is necessary to decide implementation details or current library behavior.

Prefer official documentation and specific section anchors.

### Phase 5: Generate implementation plan

Read `references/output-template.md` and use it as the output format when available.

The plan should include:

- goal and acceptance criteria
- relevant context and mandatory reading
- design calibration for Tier 2/3 work
- step-by-step implementation tasks ordered by dependency
- exact file paths and symbols when known
- testing tasks structured by scope/requirements, not just example test cases
- validation commands that are copy-paste ready and non-interactive
- known risks and approval-needed decisions

## Quality criteria

- [ ] Plan contains enough context for another agent to execute without this conversation.
- [ ] Design-risk tier is assigned after context scan.
- [ ] Tier 2/3 plans compare direct/simple vs more abstract shapes.
- [ ] Chosen design avoids both speculative architecture and careless under-design.
- [ ] Complexity ownership is explicit.
- [ ] Tasks are ordered and independently testable.
- [ ] Acceptance criteria are measurable.
- [ ] Validation commands are exact.
- [ ] Plan does not repeat obvious code facts or mandate noisy comments.

## Confirmation

After creating the plan, confirm:

- Feature name created.
- Plan saved to the exact path.
- Design-risk tier assigned.
- Approval-needed decisions called out, if any.
- Validation commands included.
- Another agent could execute this without context.

Next step:

```text
/do-plan-execution .agent-config/user/artifacts/plans/[the-plan-file].md
```
