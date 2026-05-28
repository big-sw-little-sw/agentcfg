---
name: do-mr-fix
description: Fetch and fix reviewer comments from a GitLab merge request, push updates, and reply to discussions. Use when addressing MR feedback, fixing MR review comments, or responding to MR discussions.
argument-hint: <mr-id|branch|url> [scope]
allowed-tools: Read, Write, Edit, Bash(glab:*), Bash(git:*), Bash(uv run ruff:*), Bash(uv run pyright:*), Bash(uv run pytest:*)
---

# MR Fix: Address Reviewer Comments in GitLab MR

Fetch reviewer comments from a GitLab merge request, fix the issues, push updates, and reply to discussions.


## Design Guardrail

Keep fixes focused, but do not sneak in a redesign.

If fixing an issue requires a public API change, schema/persistence change, auth/security change, major module-boundary change, new shared abstraction, or broad behavior change, pause and run `do-design-pass` instead of folding that redesign into the fix silently.

If implementation friction reveals that the original fix plan is awkward, leaky, flag-heavy, duplication-prone, or over/under-designed, emit a short design note or Plan Deviation Notice as appropriate.

After non-trivial fixes, run `do-maintainability-check`.


## Arguments

**Arguments provided**: $ARGUMENTS

**Input handling:**
- **First argument** (required): MR identifier — accepts MR number, branch name, or MR URL
- **Second argument** (optional): Scope filter to limit which comments to address

**Examples:**
- `/do-mr-fix 42` — Fix all review comments on MR #42
- `/do-mr-fix feature-branch` — Fix comments on MR for branch
- `/do-mr-fix 42 "critical only"` — Fix only critical issues
- `/do-mr-fix 42 "the race condition discussion"` — Fix a specific discussion

## Phase 1: Gather MR Context and Comments

### 1.1 Fetch MR Metadata

```bash
glab mr view <mr-identifier> -F json
```

Extract: `iid`, `source_branch`, `target_branch`, `title`.

### 1.2 Checkout MR Branch

```bash
glab mr checkout <mr-identifier>
```

### 1.3 Fetch All Discussions

```bash
glab api projects/:id/merge_requests/<iid>/discussions --paginate
```

Returns a JSON array of discussions. Key structure:
- `id`: Discussion ID (needed for replies)
- `notes[]`: Array of notes in the discussion
  - `notes[].body`: Comment text
  - `notes[].author.username`: Who wrote it
  - `notes[].resolved`: Whether resolved
  - `notes[].resolvable`: Whether it can be resolved
  - `notes[].position`: Inline position (if applicable)
    - `position.new_path`: File path
    - `position.new_line`: Line number (new side)
    - `position.old_path`, `position.old_line`: Old side references
- `notes[0]`: Root comment of the discussion

### 1.4 Filter Relevant Comments

From the discussions, extract actionable review comments.

**Include:**
- Unresolved discussions with inline positions (code-level feedback)
- Unresolved general discussions that request changes
- Comments from reviewers (not the MR author's own comments)

**Exclude:**
- Already resolved discussions
- System notes (merge status, label changes, pipeline results, etc.)
- The MR author's own questions/replies

### 1.5 Apply Scope Filter

If scope argument provided:
- Parse the scope description
- Filter comments to match
- Report: "Filtering to X out of Y actionable comments based on scope: [scope]"

### 1.6 Organize and Prioritize

Group comments by:
1. **File** — fix all comments in one file before moving to the next
2. **Line order** — top to bottom within each file

Infer severity from comment content:
- Security/correctness concerns → critical/high
- Logic questions/bugs → medium
- Style/naming suggestions → low
- Questions/clarifications → info

**Display the plan and wait for user confirmation before proceeding.** The user may:
- Approve the full plan
- Narrow the scope (skip certain comments)
- Reorder priorities
- Cancel entirely

**Do NOT begin fixing until the user confirms.**

## Phase 2: Fix Issues

For each comment, follow the fix methodology from the `do-code-review-fix` skill for full details:

### 2.1 Understand the Comment

Read the full discussion (root comment + replies) to understand:
- What the reviewer is asking for
- Any context from follow-up discussion
- Whether there's a specific suggestion to follow

### 2.2 Read Current Code

Read the file at the referenced location. If line numbers shifted since the review:
- Search for the code pattern described
- If the issue appears already fixed, note it and skip

### 2.3 Implement the Fix

- Follow the reviewer's suggestion if appropriate
- Or implement a better fix — note why in the discussion reply later
- Maintain existing code patterns and conventions

### 2.4 Verify

For critical/complex fixes:
- Run type checker on modified files
- Run relevant tests
- Run linter on modified files

## Phase 3: Validate and Confirm Push

### 3.1 Run Full Validation

After all fixes, run the validation commands from `AGENTS.md` to ensure nothing broke. Fix regressions before proceeding.

### 3.2 Confirm Commit and Push

Present to the user:
- Summary of all files modified and what changed
- The commit message to be used
- Confirmation that validation passed

**Wait for user confirmation before committing and pushing.**

Once confirmed:

```bash
git add .
git commit -m "fix: address MR review feedback

Fixes reviewer comments on !<iid>:
- [brief list of what was fixed]"
git push
```

Follow project commit conventions (Conventional Commits).

## Phase 4: Reply to Discussions

### 4.1 Prepare Replies

For each discussion that was addressed, prepare a reply. Show all replies to the user:

| Discussion | File / Line | Reply |
|------------|-------------|-------|
| `<discussion_id_short>` | `file.py:42` | Fixed in `<sha>` |
| ... | ... | ... |

**Reply formats:**
- Straightforward fix: `"Fixed in <commit_sha_short>"`
- Fix differs from suggestion: `"Fixed in <commit_sha_short> — <brief explanation of approach taken>"`
- Issue no longer applies: `"No longer applicable: <reason>"`
- Disagree with suggestion: `"I chose to keep the current approach because <reason>. Happy to discuss further."`

### 4.2 Confirm and Post Replies

**Wait for user confirmation before posting replies.** The user may edit individual replies or skip some.

Once confirmed, post each reply:

```bash
glab api projects/:id/merge_requests/<iid>/discussions/<discussion_id>/notes \
  -X POST \
  -f "body=<reply_text>"
```

**Do NOT auto-resolve discussions** — let the reviewer confirm and resolve.

## Phase 5: Summary

```text
MR fix complete
- MR: !<iid> — <title>
- Branch: <source_branch>
- Comments addressed: total X, fixed Y, already resolved Z, skipped W
- Files modified:
  - path/to/file1.py (2 comments addressed)
  - path/to/file2.py (1 comment addressed)
- Discussion replies: Y posted
- Push status: changes pushed to <source_branch>
```

## Guidelines

- **Fix one comment at a time**, completely, before moving to the next
- **Read the full discussion** — context from follow-up replies matters
- **Don't over-engineer** — fix what the reviewer asked, not more
- **If you disagree**, explain why in the discussion reply rather than silently ignoring it
- **Don't auto-resolve** discussions — let the reviewer close them
- Follow `do-code-review-fix` principles for fix quality
