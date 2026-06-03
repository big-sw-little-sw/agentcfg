---
name: do-code-review
description: Review code changes for correctness, bugs, security issues, edge cases, validation gaps, and technical defects. Use design review separately for strategic maintainability, abstraction, and module-boundary concerns.
argument-hint: commit-hash or commit-range (optional - reviews uncommitted changes if not specified)
allowed-tools: Read, Bash(git:*), Bash(*)
---

# Code Review: Find Bugs and Quality Issues

Your job is to review code changes and provide actionable feedback on **bugs and technical quality** — NOT plan adherence, strategic decisions, or style nitpicking.


## Review Routing

Use this skill for correctness and technical defects: bugs, security issues, edge cases, validation gaps, broken error handling, performance risks, and egregious standards violations that affect correctness or maintainability.

Use `do-design-review` separately when the main concern is strategic maintainability: public APIs, shared abstractions, module boundaries, duplicated domain rules, over/under-design, leaky interfaces, or misplaced complexity ownership.

For a generic "review this diff/PR" request, run this correctness review first. Add the design-review lens only when the diff is Tier 2/3 or the user asks for full/design/maintainability review.

Design discipline bridge:
- Choose the smallest design that survives the next plausible change.
- Avoid speculative architecture and careless under-design.
- Abstract shared meaning, not shared shape.
- Report design concerns only when they predict future human maintenance pain.


## Design risk tiers

- Tier 0: mechanical or purely local; implement directly.
- Tier 1: local but maintainability-relevant; implement directly and run a lightweight maintainability check after code changes.
- Tier 2: design-impacting but reversible; inspect bounded context, use a compact design note or design lens, and proceed unless asked to pause.
- Tier 3: expensive to undo; inspect broader context, use a full design pass, and pause for approval before implementation or deviation.

Tier from the request is a guess; tier after context scan is the decision.

## Core Principles

**Be certain.** Your primary job is finding bugs.
- If you're going to call something a bug, be confident it actually is one
- Don't invent hypothetical problems - explain realistic scenarios where it breaks
- If you're unsure about something, say "Uncertain: [question]" rather than flagging as definite issue
- Use tools to verify before claiming something is wrong

**Don't be a zealot about style.**
- Only flag egregious standards violations, not every minor deviation
- Some violations are acceptable when they're the simplest option
- Don't complain about else statements if early returns are already used correctly
- Focus on issues that affect correctness, security, or maintainability


## Review Scope

**Arguments provided**: $ARGUMENTS

**Determining what to review:**

1. **No arguments (default)**: Review all uncommitted changes
   - Run: `git diff` for unstaged changes
   - Run: `git diff --cached` for staged changes
   - Run: `git status --short` to identify untracked files

2. **Commit hash** (e.g., `abc123`): Review that specific commit
   - Run: `git show abc123`

3. **Commit range** (e.g., `main..feature` or `HEAD~3..HEAD`): Review commits in range
   - Run: `git diff main..feature`

4. **Branch name**: Compare current branch to specified branch
   - Run: `git diff branch-name...HEAD`

Use best judgement when processing input.

## Phase 1: Gather Context

**Diffs alone are not enough.** You need context to review properly.

### 1.1 Read Project Standards

Read `AGENTS.md` if it exists, using the available file-reading mechanism for the current tool/environment.

Check for other convention files if they exist:
- CONVENTIONS.md, .editorconfig, CONTRIBUTING.md

**Goal**: Understand established patterns so you can identify when code doesn't fit
- Common abstractions and utilities
- Error handling patterns
- Type annotation expectations
- Testing approach

**Note**: You're looking for context, not creating a checklist. Don't flag every minor deviation from AGENTS.md - only egregious violations that affect correctness or maintainability.

## Phase 2: Identify What Changed

### 2.1 Get Change Statistics

If reviewing uncommitted changes:
```bash
git status --short
git diff --stat
git diff --numstat
git diff --cached --stat
git diff --cached --numstat
```

If reviewing specific commit(s):
```bash
git show --stat $ARGUMENTS
git show --numstat $ARGUMENTS
# Or for commit range:
git diff --stat $ARGUMENTS
git diff --numstat $ARGUMENTS
```

### 2.2 List All Changed Files

Get the full list of files to review:

If uncommitted:
```bash
# Modified (unstaged)
git diff --name-only
# Modified (staged)
git diff --cached --name-only
# New files
git ls-files --others --exclude-standard
```

If specific commit(s):
```bash
git diff --name-only $ARGUMENTS
# Or for single commit:
git show --name-only --format="" $ARGUMENTS
```

### 2.3 Triage Changed Files (Avoid Noise)

Not every changed file deserves a full-file read.

**Goal**: Spend attention where bugs live (source + tests), and avoid wasting time on large/autogenerated/lock artifacts.

Classify each changed file into one of these buckets:

**A) Full review (read file + related context as needed)**
- Source code: `*.py`, `*.ts`, `*.tsx`, `*.js`, `*.jsx`, `*.rs`, `*.go`, `*.java`, `*.c`, `*.cpp`
- Tests: `test_*`, `*_test`, `tests/**`, `__tests__/**`

**B) Light review (diff-first; read file only if diff is non-trivial)**
- Config: `pyproject.toml`, `Cargo.toml`, `package.json`, `go.mod`, `pom.xml`, `*.yml`, `*.yaml`, `*.toml`, `*.json` (non-lock)
- Docs: `*.md`

**C) Diff-only (do NOT read entire file)**
- Lockfiles and dependency snapshots: `uv.lock`, `poetry.lock`, `pnpm-lock.yaml`, `yarn.lock`, `package-lock.json`, `Cargo.lock`, `go.sum`
- Generated/minified assets: `*.min.js`, `*.map`

**D) Skip (mention in report if changed, but don’t analyze contents)**
- Binaries and large blobs: `*.png`, `*.jpg`, `*.pdf`, `*.zip`, etc.
- Vendored/build outputs: `node_modules/**`, `dist/**`, `build/**`, `.venv/**`, `target/**` (unless intentionally committed)

**Heuristic guardrails:**
- If `git diff --numstat` shows `-` for a file (binary), treat as bucket D.
- If a file is very large (e.g. >2k lines or >200KB), treat as bucket C unless it’s core source code and the diff is small/targeted.
- If you used bucket C/D due to size/noise, explicitly call that out in the report so the reader knows it was **diff-only** (or skipped) and why.
- For bucket C, review the *diff* for unexpected dependency changes (new packages, major bumps, removed constraints), but don’t parse the whole file.

## Phase 3: Review Each File

For **each bucket A/B file**, analyze for bugs and quality issues. For bucket C/D, do a targeted check per the triage rules.

### 3.1 Read Full File Context

**CRITICAL**: Don’t review changes in isolation, but also don’t blindly read huge/irrelevant files.

- Use the diff to identify which files changed
- For bucket A: read the complete file (unless very large; then use targeted reads around the changed region)
- For bucket B: start with the diff; read the file only if the diff is non-trivial or suggests risk
- For bucket C: review the diff only
- Read related files (imports, base classes) if needed
- Code that looks wrong in isolation may be correct given surrounding logic - and vice versa

**Only review the changes** - don't review pre-existing code that wasn't modified.

### 3.2 What to Look For

Focus on issues in order of priority:

#### 1. Bugs (PRIMARY FOCUS)

**Logic Errors:**
- Off-by-one errors
- Incorrect conditionals or boolean logic
- If-else guards: missing guards, incorrect branching, unreachable code paths
- Incorrect assumptions or invariants

**Edge Cases:**
- Null/empty/undefined inputs
- Error conditions not handled
- Race conditions or concurrency issues
- Boundary conditions

**Error Handling:**
- Broken error handling that swallows failures
- Functions that throw unexpectedly
- Error types that aren't caught by caller
- Missing try/except around fallible operations

**Security Issues (flag as CRITICAL):**
- SQL injection, command injection, path traversal
- Auth bypass or missing auth/authorization checks
- Data exposure (secrets, credentials, PII)
- Unsafe deserialization

#### 2. Performance (Only if Obviously Problematic)

- O(n²) on unbounded data
- N+1 query patterns
- Blocking I/O on hot paths
- Excessive memory usage or leaks

#### 3. Structure and Patterns (Only When Technically Material)

Use this section to catch defects caused by ignoring codebase structure, not to perform a full design review.

Flag only when:
- The change bypasses an established utility or abstraction and is likely to duplicate a bug-prone invariant, policy, or validation rule.
- The change violates an architectural boundary in a way that can cause incorrect behavior, security risk, or fragile coupling.
- A changed function has enough branching or mixed responsibility that correctness is hard to verify.
- Naming, types, or API shape are misleading enough to cause realistic misuse.
- Missing types/type hints would have prevented a realistic bug or API misuse.

If the concern is mainly long-term maintainability, abstraction shape, or module boundaries, route it to `do-design-review` instead of expanding this review.

#### 4. Standards Violations (Only Egregious and Material)

Do not flag standards as a checklist. Flag a standards violation only when all are true:
- It clearly violates `AGENTS.md` or a local convention evidenced in nearby code.
- It affects correctness, security, validation, type safety, contract behavior, or meaningful maintainability.
- It is specific and actionable.

Examples of what not to flag:
- Minor formatting that standard tools would catch.
- Preference-only naming or style differences.
- Missing prose/docstring on trivial self-documenting code.
- A convention that is not documented and not consistently followed locally.

Examples of what to flag:
- Documented validation/security/persistence boundary is bypassed.
- Documented error-handling or domain invariant is ignored.
- API/type contract is violated across a boundary.
- Required validation/test command from `AGENTS.md` is skipped for a relevant change.

## Phase 4: Verify Before Flagging

**Be certain before calling something a bug.**

### 4.1 Use Tools to Verify

When you suspect an issue, verify it using the commands defined in **`AGENTS.md`**:

**Type errors:**
Run the type checking command specified in `AGENTS.md`.

**Code quality issues:**
Run the linting command specified in `AGENTS.md`.

**Logic errors:**
Run the test command specified in `AGENTS.md`.

### 4.2 Check Existing Code

Use the available file-reading mechanism to check how existing code handles similar problems:
- How do other parts of the codebase handle this pattern?
- Are there utility functions that should be used?
- Is this actually a bug or just a different approach?

### 4.3 When Uncertain

If you can't verify something with tools or existing code:
- **Don't flag it as a definite issue**
- Instead, note it as: "Uncertain: [question about potential issue]"
- Explain what you'd need to verify it

**Example:**
```
Severity: info
Issue: Uncertain: Possible race condition in concurrent writes
Detail: The code writes to shared state without locking. I couldn't find
other examples of concurrent access in the codebase to compare against.
If this function can be called concurrently, this could cause data corruption.
```

## Phase 5: Generate Review Report

**Determine filename based on what's being reviewed:**

The filename should be descriptive of the changes being reviewed, not generic.

**Filename pattern**: `.agent-config/user/artifacts/code-reviews/[descriptive-name]-[YYYY-MM-DD-HHMM].md`

**How to create descriptive name:**

1. **For uncommitted changes (staged + unstaged)**:
   - Analyze the changes and create a short kebab-case description (2-4 words)
   - Based on git diff context: what feature/fix/refactor is being worked on?
   - Examples:
     - `add-user-authentication-2026-01-23-1430.md`
     - `fix-memory-leak-executor-2026-01-23-1430.md`
     - `refactor-storage-layer-2026-01-23-1430.md`
     - `update-api-docs-2026-01-23-1430.md`
   - If truly mixed/unclear changes: `working-tree-2026-01-23-1430.md`

2. **For specific commit**:
   - Use the commit message subject as base for the description
   - Convert to kebab-case, keep it short (2-5 words)
   - Examples:
     - For commit "feat(auth): add JWT authentication": `add-jwt-authentication-2026-01-23-1430.md`
     - For commit "fix: resolve race condition in executor": `fix-race-condition-executor-2026-01-23-1430.md`
     - For commit "refactor: simplify storage paths": `simplify-storage-paths-2026-01-23-1430.md`

3. **For commit range**:
   - Describe the overall theme/purpose of the commits
   - Examples:
     - For `main..feature-auth`: `auth-feature-branch-2026-01-23-1430.md`
     - For `HEAD~3..HEAD`: `last-3-commits-2026-01-23-1430.md` (if no clear theme)
     - For range with clear theme: `api-refactor-series-2026-01-23-1430.md`

**Guidelines:**
- Keep descriptive part under 40 characters
- Use kebab-case (lowercase with hyphens)
- Be specific but concise
- Focus on what changed, not review type

**Timestamp format**: Use `YYYY-MM-DD-HHMM` (24-hour time, no seconds)

**Collision handling**: If the generated filename already exists, append an incrementing suffix:
- First collision: add `-2` (e.g., `[descriptive-name]-2.md`)
- Second collision: add `-3` (e.g., `[descriptive-name]-3.md`)
- Continue incrementing until finding an available filename

**Examples:**
- `add-user-authentication-2026-01-23-1430.md` → `add-user-authentication-2026-01-23-1430-2.md`
- `fix-race-condition-2026-01-23-1430.md` → `fix-race-condition-2026-01-23-1430-2.md`

Check if file exists before writing and increment as needed.

### Report Format

```markdown
# Code Review Report

**Date**: [YYYY-MM-DD HH:MM]
**Reviewed**: [uncommitted changes | commit abc123 | range main..feature]

## Stats

- Files Modified: X
- Files Added: Y
- Files Deleted: Z
- Lines Added: +N
- Lines Removed: -M

## Review Coverage

- Full review (read file): [paths]
- Light review (diff-first): [paths]
- Diff-only (by policy/size): [paths + short reason]
- Skipped (binary/vendor/build): [paths + short reason]

## Summary

[Brief 2-3 sentence overview of changes and overall assessment]

## Issues Found

[If no issues: "Code review passed. No technical issues detected."]

[For each issue:]

---

**Severity**: critical | high | medium | low | info

**File**: `path/to/file.py`

**Line**: 42

**Issue**: [One-line description]

**Detail**: [Explanation of why this is a problem and its impact]

**Suggestion**: [Specific, actionable fix with code example if applicable]

---

## Validation Results

[Only include if validation commands were run]

### Type Checking
[Output if run]

### Linting
[Output if run]

### Tests
[Output if run]

## Recommendations

[High-level recommendations for improving code quality, if applicable]

---

*Review completed by code-review agent*
```

### Severity Guidelines

Use these severity levels based on impact:

**critical**:
- Security vulnerabilities (injection, auth bypass, data exposure)
- Data corruption risks
- Production-breaking bugs that will cause immediate failures

**high**:
- Logic errors that cause incorrect behavior
- Broken error handling that masks failures
- Significant performance issues (O(n²) on unbounded data)

**medium**:
- Code quality issues affecting maintainability
- Minor bugs that occur in edge cases
- Missing error handling for non-critical paths

**low**:
- Egregious style violations (only if they violate documented standards)
- Small optimizations
- Minor inconsistencies

**info**:
- Suggestions for improvement
- Uncertain issues that need verification
- Questions about potential problems

**Clearly communicate scenarios where issues manifest:**
- Don't just say "this could break" - explain when/how it breaks
- If severity depends on usage context, state the conditions explicitly
- Example: "This causes N+1 queries if dataset has >100 versions (typical production size)"

### Output Requirements

1. **Be specific**: Always include file paths and line numbers
2. **Explain impact**: Why is this a problem? When does it break? What's the consequence?
3. **Provide context**: If it depends on specific scenarios, state them explicitly
4. **Suggest fixes**: Don't just complain - show how to fix it with code examples
5. **Be concise**: Write so the reader can quickly understand without reading too closely
6. **Don't overstate**: Match severity to actual impact
7. **Matter-of-fact tone**: No flattery, no accusation, just facts

### What NOT to Include

- Praise like "Great job!", "Nice work!", "Thanks for..."
- Hypothetical problems without realistic scenarios
- Style nitpicks that don't affect correctness
- Issues that standard tools would catch automatically (unless they indicate deeper problems)
- Vague complaints without specific file/line references

## Phase 6: Display Summary

After saving the report, display a concise summary to the user:

```text
Code review complete
- Reviewed: X files, +N / -M lines
- Issues: critical A, high B, medium C, low D, info E
- Report: .agent-config/user/artifacts/code-reviews/review-[timestamp].md

[If no issues:] No technical issues detected.
[If issues found:] Review the report for detailed findings and recommendations.
```

## Important Reminders

**Before flagging anything:**
- Be certain it's actually a bug or quality issue
- Verify with tools when possible
- Check existing code for similar patterns
- If unsure, mark as "Uncertain" with info severity

**Focus priorities:**
- 1. Bugs: logic errors, security, broken error handling.
- 2. Performance: only if obviously problematic.
- 3. Structure: only when technically material.
- 4. Standards: only egregious and material violations.

**Don't:**
- Review pre-existing code that wasn't changed
- Invent hypothetical problems without realistic scenarios
- Flag style preferences or minor deviations
- Be overly zealous about standards - pragmatism matters
- Run extra searches beyond understanding changed files

## Examples of Good Findings

### Example 1: Clear Bug with Context
```
Severity: high
File: packages/server/src/hive/services/sync.py
Line: 127
Issue: Missing error handling for IntegrityError during version creation

Detail: The create_version() call at line 127 can raise IntegrityError if a version
with this ID already exists (this happens when retrying a failed sync operation).
Without catching this exception, the entire operation fails without cleanup,
potentially leaving stale locks that block subsequent syncs.

Suggestion: Wrap in try/except and handle the duplicate version case:
  try:
      version = await catalog.create_version(dataset_id, version_data)
  except IntegrityError:
      existing = await catalog.get_version(dataset_id, version_id)
      if existing.state == "committing":
          # Resume existing operation
          return existing
      raise VersionAlreadyExistsError(...)
```

### Example 2: Performance Issue with Realistic Impact
```
Severity: high
File: packages/sdk/src/hive_sdk/operations.py
Line: 89
Issue: N+1 query pattern when fetching version metadata

Detail: The code iterates over versions (line 89) and makes a separate API call
for each version's metadata (line 92). For datasets with >100 versions (typical
in production), this causes 100+ sequential HTTP requests, taking 10-30 seconds
instead of <1 second with batch fetching.

Suggestion: Use the batch metadata endpoint:
  # Instead of:
  for v in versions:
      metadata = client.get_version_metadata(v.id)

  # Use:
  version_ids = [v.id for v in versions]
  metadata_map = client.get_batch_metadata(version_ids)
```

### Example 3: Uncertain Issue
```
Severity: info
File: packages/server/src/hive/executors/commit.py
Line: 234
Issue: Uncertain: Possible race condition in concurrent commit attempts

Detail: The code checks if a commit marker exists (line 234), then writes a new one
(line 238) without any locking mechanism. I couldn't find other examples of concurrent
commit handling to verify the pattern.

If two commit operations for the same version run simultaneously, they could both
pass the existence check and create conflicting markers, violating the "single commit
sequence" invariant mentioned in AGENTS.md §3.4.

Suggest verifying: Is this path protected by locks at a higher level? If not, should
this use atomic file creation (O_EXCL) or database-level locking?
```

### Example 4: Standards Violation That Affects Correctness
```
Severity: medium
File: packages/server/src/hive/executors/sync_commit.py
Line: 156
Issue: Violates "facts before visibility" invariant

Detail: AGENTS.md §3.4 states "Facts-before-visibility: only mark `completed` after
commit marker exists." This code marks the intent as completed (line 156) before
writing the commit marker (line 162).

If the process crashes between these lines, the database shows a completed operation
but NFS has no commit marker, breaking the durability guarantee. Clients would see
a "successful" commit that can't be verified.

Suggestion: Reverse the order - write marker first, then update DB:
  await storage.write_commit_marker(marker_path, marker_data)  # Line 162 first
  await catalog.complete_intent(intent_id)  # Line 156 second
```

## Examples of Bad Findings

### Bad: Too Vague
```
Severity: medium
File: some_file.py
Line: 50
Issue: Code could be better
Detail: This isn't very good
Suggestion: Improve it
```

### Bad: Style Nitpick Without Real Impact
```
Severity: low
File: packages/server/src/hive/models/version.py
Line: 34
Issue: Function is 55 lines, should be under 50
Detail: AGENTS.md suggests functions under 50 lines as a guideline
Suggestion: Split into smaller functions
```
*Why bad: The 50-line guideline is a suggestion, not a hard rule. Only flag if
the function is actually doing too many things or hard to understand.*

### Bad: Hypothetical Problem
```
Severity: high
File: packages/sdk/src/hive_sdk/client.py
Line: 89
Issue: Missing timeout could cause infinite hang
Detail: This HTTP request has no timeout
Suggestion: Add a timeout
```
*Why bad: Doesn't check if httpx client already has default timeout configured.
Verify first before claiming it's missing.*

### Bad: Overstated Severity
```
Severity: critical
File: packages/server/src/hive/api/datasets.py
Line: 45
Issue: Variable name is unclear
Detail: The variable 'x' should have a more descriptive name
Suggestion: Rename to 'dataset_count'
```
*Why bad: Unclear variable names are at most "low" severity, not "critical".
Critical is for security vulnerabilities and production-breaking bugs.*

---
