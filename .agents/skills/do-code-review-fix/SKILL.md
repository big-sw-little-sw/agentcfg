---
name: do-code-review-fix
description: Fix bugs and issues found in a code review report
argument-hint: review-file [scope]
allowed-tools: Read, Write, Edit, Bash(uv run ruff:*), Bash(uv run pyright:*), Bash(uv run pytest:*), Bash(git:*)
---

# Fix Code Review: Address Issues from Review Report

Process and fix bugs and issues identified in a code review report.


## Design Guardrail

Keep fixes focused, but do not sneak in a redesign.

If fixing an issue requires a public API change, schema/persistence change, auth/security change, major module-boundary change, new shared abstraction, or broad behavior change, pause and run `do-design-pass` instead of folding that redesign into the fix silently.

If implementation friction reveals that the original fix plan is awkward, leaky, flag-heavy, duplication-prone, or over/under-designed, emit a short design note or Plan Deviation Notice as appropriate.

After non-trivial fixes, run `do-maintainability-check`.


## Arguments

**Arguments provided**: $ARGUMENTS

**Input handling:**
- **First argument** (required): Path to review report file
  - Full path: `.agent-config/user/artifacts/code-reviews/add-user-auth-2026-01-23-1430.md`
  - Or relative: `add-user-auth-2026-01-23-1430.md` (will look in `.agent-config/user/artifacts/code-reviews/`)
- **Second argument** (optional): Scope filter (free-form text)
  - Limits which issues to fix from the review
  - Examples: "critical only", "auth issues", "just the race condition", "everything except style"

**Examples:**
- `/do-code-review-fix auth-review-2026-01-23.md` - Fix all issues in report
- `/do-code-review-fix auth-review-2026-01-23.md "critical and high only"` - Fix only critical/high severity
- `/do-code-review-fix auth-review-2026-01-23.md "just the SQL injection issue"` - Fix specific issue

## Phase 1: Read and Understand Review Report

### 1.1 Read the Complete Review

Read the entire review file to understand all issues.

### 1.2 Extract All Issues

Parse the review and identify:
- Total number of issues found
- Severity breakdown (critical, high, medium, low, info)
- Files affected
- Line numbers for each issue

### 1.3 Apply Scope Filter

If scope argument provided, filter issues to fix:
- Parse the scope description
- Determine which issues match the scope
- Report: "Filtering to X out of Y total issues based on scope: [scope]"

If no scope provided, fix all issues.

### 1.4 Prioritize Fixes

**Fix order by severity:**
1. Critical (security vulnerabilities, data corruption, production-breaking bugs)
2. High (logic errors, broken error handling)
3. Medium (code quality, minor bugs)
4. Low (style violations, small optimizations)
5. Info (suggestions - ask before fixing these)

**Within same severity:**
- Fix by file (complete all fixes in one file before moving to next)
- Fix in line number order (top to bottom)

### 1.5 Create Fix Plan

Before starting fixes, create a clear plan:

**For each issue in scope, note:**
- File and line number
- Severity
- Issue description
- Suggested fix from review
- Dependencies (does this fix depend on another fix first?)

**Display plan to user** before proceeding.

## Phase 2: Fix Issues One by One

For **each issue** in priority order:

### 2.1 Explain the Issue

Before fixing, clearly explain:
- **What was wrong**: Describe the bug/issue
- **Why it's a problem**: Explain the impact
- **What we'll do**: Describe the fix approach

### 2.2 Read Current Code

Read the file containing the issue to understand context.

If the file has changed since the review:
- Check if the issue still exists at the reported line
- If line numbers shifted, find the issue by code pattern
- If issue is already fixed, note it and skip

### 2.3 Implement the Fix

Apply the fix:
- Follow the suggested fix from the review (if appropriate)
- Or implement a better fix if you identify one
- Follow AGENTS.md standards
- Maintain existing code style and patterns

**Show the fix:**
- Display the old code (what was wrong)
- Display the new code (what was fixed)
- Explain what changed

### 2.4 Verify the Fix (Optional)

For critical or complex fixes, consider running quick verification before moving on:

**For logic errors/bugs:**
- Run relevant tests to verify the fix works

**For type errors:**
- Run pyright on the file to verify types pass

**For security issues:**
- Verify the vulnerability is closed
- Consider adding test case that would have caught the vulnerability

**For simpler fixes:**
- Can defer verification to Phase 3 full validation
- Faster to fix multiple issues then validate once

### 2.5 Handle Dependencies

If fixing this issue reveals or requires other changes:
- Note the dependency
- Fix prerequisites first
- Return to original fix

## Phase 3: Validate All Fixes

After **all in-scope issues** are fixed:

### 3.1 Run Full Validation

Run the relevant validation commands from local project guidance and tooling:

- Check `AGENTS.md` for explicit validation commands.
- Check `pyproject.toml`, package scripts, Makefiles, tox/nox configs, and nearby CI config.
- Prefer the smallest command that covers the changed behavior, then run broader validation when the blast radius warrants it.

### 3.2 Fix Any Regressions

If validation fails:
- Identify what broke
- Fix the regression
- Re-run validation
- Continue until all validation passes

## Phase 4: Summary Report

After all fixes complete and validation passes, provide summary:

### Fixes Applied

**Review file processed**: 
- `.agent-config/user/artifacts/code-reviews/[filename].md`

**Scope applied** (if any):
- "[scope description]"

**Issues in review**: X total
**Issues fixed**: Y issues

**Breakdown by severity:**
- Critical: X issues fixed
- High: Y issues fixed
- Medium: Z issues fixed
- Low: A issues fixed
- Info: B issues (or skipped)

**Issues not in scope** (if scope filter applied):
- [severity] file.py:123 - Brief description
- [severity] file.py:456 - Brief description
- (Or "None - all issues addressed")

**Files modified:**
- `path/to/file1.py` (3 issues fixed)
- `path/to/file2.py` (2 issues fixed)

**Tests added/updated:**
- `tests/path/to/test_file1.py` (new test for bug fix)
- `tests/path/to/test_file2.py` (updated existing test)

### Validation Results

Reference the validation report output.

### Ready for Commit

All fixes applied and validated. Use `/do-commit` to create a commit.

## Important Guidelines

**Be methodical:**
- Fix one issue completely before moving to the next
- Explain each fix clearly before implementing
- Verify each fix with tests or validation
- Don't skip critical or high severity issues

**Handle uncertainties:**
- ❓ If a suggested fix seems wrong, explain why and propose alternative
- ❓ If an issue is unclear, read surrounding code for context
- ❓ If you can't reproduce the issue, note it and explain

**Maintain quality:**
- Follow existing code patterns in the file
- Respect AGENTS.md standards
- Keep fixes minimal and focused
- Don't introduce new issues while fixing old ones

**Communication:**
- Show progress as you work through issues
- Explain what you're doing at each step
- Ask for clarification if review suggestions are ambiguous
- Report if issues are already fixed or no longer present

## Edge Cases

**Issue already fixed:**
- Note that issue is resolved
- Skip to next issue
- Include in summary report

**Suggested fix won't work:**
- Explain why the suggested fix is problematic
- Propose alternative fix
- Implement better solution

**Can't locate issue (line numbers shifted):**
- Search for the code pattern described in review
- If found, fix at new location
- If not found, note as "possibly already fixed"

**Multiple issues on same line:**
- Fix all issues at that line together
- Test all related changes together

**Breaking change required:**
- Note the breaking change
- Explain the necessity
- Update tests and documentation
- Call out in summary

---
