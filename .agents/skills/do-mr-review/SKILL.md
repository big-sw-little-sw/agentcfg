---
name: do-mr-review
description: Review a GitLab merge request for correctness and technical defects, with design-review lens only for full/design/maintainability review or Tier 2/3 maintainability risk.
argument-hint: <mr-id|branch|url> (required)
allowed-tools: Read, Bash(glab:*), Bash(git:*), Bash(uv run ruff:*), Bash(uv run pyright:*), Bash(uv run pytest:*)
---

# MR Review: Review GitLab Merge Request

Review code changes in a GitLab merge request, post findings as inline comments, and optionally approve.


## Review Routing

For normal MR review, focus on correctness, bugs, security, edge cases, validation gaps, and technical defects.

If the user asks for full review, design review, or maintainability review, include the `do-design-review` lens before posting comments. Also include the design lens when the MR is Tier 2/3: public APIs, shared abstractions, module boundaries, persistence/schema/config/auth changes, or behavior used by multiple callers.

Only post design comments when they predict future human maintenance pain, not as style preferences.


## Design risk tiers

- Tier 0: mechanical or purely local; implement directly.
- Tier 1: local but maintainability-relevant; implement directly and run a lightweight maintainability check after code changes.
- Tier 2: design-impacting but reversible; inspect bounded context, use a compact design note or design lens, and proceed unless asked to pause.
- Tier 3: expensive to undo; inspect broader context, use a full design pass, and pause for approval before implementation or deviation.

Tier from the request is a guess; tier after context scan is the decision.

## Arguments

**Arguments provided**: $ARGUMENTS

**Input**: MR identifier (required) — accepts MR number, branch name, or MR URL.

**Examples:**
- `/do-mr-review 42`
- `/do-mr-review feature-branch`
- `/do-mr-review https://gitlab.com/group/project/-/merge_requests/42`

## Phase 1: Gather MR Context

### 1.1 Fetch MR Metadata

```bash
glab mr view $ARGUMENTS -F json
```

Extract and store these fields (needed throughout):
- `iid`: MR internal ID (used in all API calls)
- `title`, `description`: MR context
- `source_branch`, `target_branch`: branches involved
- `diff_refs.base_sha`, `diff_refs.head_sha`, `diff_refs.start_sha`: required for inline comments
- `author.username`: MR author

### 1.2 Checkout MR Branch

```bash
glab mr checkout $ARGUMENTS
```

Enables full file reads, running tests, and type checking locally.

### 1.3 Get the Diff

```bash
glab mr diff $ARGUMENTS --color=never
```

This is the basis for review — shows changes relative to the target branch.

### 1.4 Fetch Existing Discussions

```bash
glab api projects/:id/merge_requests/<iid>/discussions --paginate
```

Collect all existing discussions (resolved and unresolved). For each, note:
- Discussion ID
- File path and line number (if inline)
- Issue description (from `notes[0].body`)
- Whether resolved

This is used in Phase 3 to deduplicate findings and avoid re-posting issues already flagged in previous review rounds.

## Phase 2: Review the Code

Follow the review methodology from the `do-code-review` skill for full details:

### 2.1 Read Project Standards

Read `AGENTS.md` if present.

### 2.2 Triage Changed Files

From the diff, classify files:
- **Full review**: Source code, tests — read complete file
- **Light review**: Config, docs — diff-first, read if non-trivial
- **Diff-only**: Lock files, generated artifacts
- **Skip**: Binaries, vendored

### 2.3 Review Each File

For full-review files, read the complete file (branch is checked out locally) and analyze for:

1. **Bugs** (primary focus): Logic errors, edge cases, error handling, security
2. **Performance**: Only obviously problematic (O(n²), N+1 queries)
3. **Structure**: Deviations from codebase patterns
4. **Standards**: Only egregious violations

**Only review the MR changes**, not pre-existing code.

### 2.4 Verify Before Flagging

Use tools to confirm findings before posting:
- Type errors → run type checker per `AGENTS.md`
- Logic errors → run relevant tests
- Check existing code for similar patterns

If uncertain, use `[info]` severity with "Uncertain:" prefix.

## Phase 3: Deduplicate and Confirm Findings

Before posting anything to the MR, cross-reference findings against existing discussions and present a numbered list with smart defaults.

### 3.1 Deduplicate Against Existing Discussions

For each finding, check if a substantially similar discussion already exists on the MR (same file, same or nearby line, same issue). A finding is a **duplicate** if an existing discussion covers the same problem, even if wording differs.

### 3.2 Present Findings with Smart Defaults

Display findings in two groups:

```
## Review Findings for !<iid>

### Will Post (new):
 1. [severity] file.py:42 — Issue description
 2. [severity] file.py:89 — Issue description

### Will Skip (already flagged):
 3. [severity] file.py:30 — Issue description
                             Existing discussion: <discussion_url_or_id>
 4. [severity] other.py:15 — Issue description
                              Existing discussion: <discussion_url_or_id>

Confirm to post 1–2, or adjust
(e.g. "skip #1", "also post #3", "edit #2 to mention X").
```

**Defaults:**
- **NEW** findings → will post
- **DUPLICATE** findings → will skip

### 3.3 Wait for User Confirmation

**Do NOT post any comments to the MR until the user confirms.** The user may:
- **"confirm"** / **"looks good"** → post all items marked "Will Post"
- **"skip #N"** → remove a specific finding
- **"also post #N"** → override a duplicate and post it anyway
- **"edit #N to ..."** → revise a finding's content, then re-present for confirmation
- **"cancel"** → post nothing

## Phase 4: Post Inline Comments

### 4.1 Comment Format

Each inline comment should follow this template:

```
**[severity]** Issue description

Detail: Explanation of the problem and when/how it manifests.

Suggestion:
```python
# suggested fix
```
```

Severity levels: `[critical]`, `[high]`, `[medium]`, `[low]`, `[info]`

### 4.2 Post Inline Comments via API

**CRITICAL**: Use JSON input (`--input -` with `-H "Content-Type: application/json"`), NOT `-f` form fields. Form fields (`-f "position[key]=value"`) create `DiscussionNote` types that appear only in the MR overview. JSON input creates `DiffNote` types that render inline on the diff.

For each finding on a specific line in a changed file:

**Step 1: Write comment body to a temp file** (handles multi-line content and special characters):
```bash
cat > /tmp/mr_comment_N.txt << 'BODY'
<formatted_comment>
BODY
```

**Step 2: JSON-encode the body and post with proper nested position object**:
```bash
BODY=$(cat /tmp/mr_comment_N.txt | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")

cat <<EOF | glab api projects/:id/merge_requests/<iid>/discussions \
  -X POST --input - -H "Content-Type: application/json"
{
  "body": $BODY,
  "position": {
    "position_type": "text",
    "base_sha": "<base_sha>",
    "head_sha": "<head_sha>",
    "start_sha": "<start_sha>",
    "old_path": "<file_path>",
    "new_path": "<file_path>",
    "new_line": <line_number>
  }
}
EOF
```

**Step 3: Verify the response** — check that the API response contains `"type":"DiffNote"`. If it shows `"type":"DiscussionNote"`, the position was not resolved and the comment will only appear in the MR overview, not inline on the diff.

**Required fields:**
- `:id` — auto-resolved by `glab` to current project
- `<iid>` — MR's `iid` from Phase 1
- `<base_sha>`, `<head_sha>`, `<start_sha>` — from `diff_refs` in Phase 1
- `old_path` and `new_path` — **both required**. For modified files these are the same path. For renamed files, `old_path` is the pre-rename path
- `<file_path>` — repo-relative path
- `<line_number>` — **must be a JSON integer** (not a quoted string), line number in the new version of the file

**For comments on deleted lines**, use `old_line` (integer) instead of `new_line`.

### 4.3 Post Summary Comment

After all inline comments are posted:

```bash
glab mr note $ARGUMENTS -m "<summary>"
```

Summary template:
```
## Code Review Summary

**Files reviewed**: X
**Issues found**: Y (critical: A, high: B, medium: C, low: D, info: E)

[If issues found:] Please address the inline comments.
[If clean:] No issues found.
```

## Phase 5: Approve or Not

If no critical or high issues were found, ask the user whether to approve:

**Wait for user confirmation before approving.** Approval is a separate decision from posting comments.

If user confirms:
```bash
glab mr approve $ARGUMENTS
```

If critical/high issues exist, recommend against approval and state this in the summary.

## Phase 6: Display Summary

```text
MR review complete
- MR: !<iid> — <title>
- Branch: <source> → <target>
- Author: @<author>
- Files reviewed: X
- New comments posted: Y
- Duplicates skipped: Z
- Issues: critical A, high B, medium C, low D, info E
- Approval: approved / not approved — address critical/high issues
```

## Guidelines

- **Be certain** before posting — inline comments are visible to the team
- **Verify with tools** when possible
- **Uncertain issues**: `[info]` severity with "Uncertain:" prefix
- **Tone**: Matter-of-fact, specific, actionable — no flattery
- Review only the MR diff, not pre-existing code
