---
name: do-system-review
description: Analyze implementation, plan adherence, deviations, maintainability failures, and human corrections to recommend concrete improvements to the agent system.
argument-hint: plan-file execution-report [reviews-or-notes]
---

# System Review

System review turns repeated agent mistakes into better guidance, examples, triggers, or workflow checks.

It is not code review. It is not design review. It is a meta-review of the agent workflow.

## Design risk tiers

- Tier 0: mechanical or purely local; implement directly.
- Tier 1: local but maintainability-relevant; implement directly and run a lightweight maintainability check after code changes.
- Tier 2: design-impacting but reversible; inspect bounded context, use a compact design note or design lens, and proceed unless asked to pause.
- Tier 3: expensive to undo; inspect broader context, use a full design pass, and pause for approval before implementation or deviation.

Tier from the request is a guess; tier after context scan is the decision.


## Inputs

Use available artifacts:

- Original request.
- Approved plan.
- Plan Deviation Notices.
- Execution report.
- Maintainability check.
- Code review findings.
- Design review findings.
- Plan-adherence review findings.
- Human corrections.

Arguments:

```text
$ARGUMENTS
```

## Failure classification

Classify each meaningful failure as one of:

Prompt/guidance gap:

- The agent lacked the right rule.

Project context gap:

- `AGENTS.md` did not explain ownership, boundaries, or contracts.

Skill workflow gap:

- The right skill did not trigger, or triggered too late.

Calibration gap:

- The agent knew the rule but chose the wrong balance.

Example gap:

- A repo-specific before/after example would teach the preference better than another rule.

One-off mistake:

- No durable system change needed.

## Design-specific failure patterns

Treat these as first-class process failures when repeated or high-impact:

- Speculative abstraction.
- Under-designed tactical patch.
- Duplicated shared meaning.
- Leaky interface.
- Misplaced complexity ownership.
- Flag/mode/special-case explosion.
- Verbose comments or cryptic comments.
- Silent plan deviation.
- Tier 3 decision made without approval.

## Noise filter

Recommend changes only when they are:

- Evidence-based.
- Preventive.
- Actionable.
- Likely to recur or high-impact enough to justify durable guidance.

Corrected mistakes become examples first; rules only after repeated evidence.

## Output format

Save your analysis to:

```text
.agent-config/user/artifacts/system-reviews/[feature-name]-review.md
```

Use this structure:

```markdown
# System Review: [feature-name]

## Overall Assessment
- Alignment score: __/10
- Main workflow failure:
- Main design-calibration failure:

## Evidence Reviewed
- Plan:
- Execution report:
- Plan-adherence review:
- Design review:
- Code review:
- Human corrections:

## Findings

### 1. [Finding title]
- Classification: prompt/guidance gap / project context gap / skill workflow gap / calibration gap / example gap / one-off mistake
- Evidence:
- Impact:
- Recommended system change:

## System Improvement Recommendations

### Project guidance patch
```markdown
[Suggested AGENTS.md addition]
```

### Skill patch
```markdown
[Suggested skill text]
```

### Calibration example
```markdown
## Design Calibration Example

Situation:
- ...

Agent version:
- ...

Preferred version:
- ...

Why this is better:
- ...

Rule taught:
- ...
```

## Do Not Change
- Guidance or workflow areas that worked well and should not be modified.
```

## Constraints

- Produce reviewable guidance patches, not vague advice.
- Do not recommend a new rule for a one-off mistake unless the impact is high.
- Prefer examples over doctrine when the issue is calibration.
- Be specific about where the change should go.
