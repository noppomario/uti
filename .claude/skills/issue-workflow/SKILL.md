---
name: issue-workflow
description: GitHub Issue-based task execution workflow with project management. Use when user says "work on issue", "implement issue", "start issue", or "/issue-workflow {issue-url}". Includes project status management, plan mode investigation, and PR auto-linking.
---

# Issue Workflow

Execute development tasks based on GitHub Issues with full project management.

## Usage

```text
/plan
/issue-workflow https://github.com/owner/repo/issues/123
```

## Prerequisites

- **Plan mode required**: Enter plan mode with `/plan` before invoking this skill
- Plan mode ensures user approval before implementation begins

## Arguments

- **Issue URL** (required): Full GitHub issue URL

ARGUMENTS: Issue URL passed from skill invocation

## Workflow Overview

### Phase 1: Initialization (Plan Mode - Read Only)

1. Parse Issue URL to extract `owner`, `repo`, `issue_number`
2. Fetch issue details: `gh issue view {url} --json title,body,labels,projectItems`
3. Extract project info from `projectItems` (save for later use)
4. Note project field IDs (see [gh-project-api.md](references/gh-project-api.md))

### Phase 2: Investigation and Planning (Plan Mode - Read Only)

1. Investigate the codebase based on issue requirements
2. Create implementation plan (write to plan file)
3. Use `ExitPlanMode` to request user approval

### Phase 3: Post-Approval Actions (After Plan Mode Exit)

After user approves the plan:

1. Post **full plan content** to Issue (not a summary):

   ```bash
   # Read the plan file and post as-is
   gh issue comment {url} --body "## Implementation Plan

   $(cat {plan_file_path})"
   ```

   **IMPORTANT**: Post the complete plan file content, not a summary.
   This preserves all decisions and prevents information loss.

2. Update project status to "In Progress" using the commands in
   [gh-project-api.md](references/gh-project-api.md)

### Phase 4: Implementation

1. Follow the plan to implement changes
2. Periodically check remaining tasks
3. Track any deviations from original plan

### Phase 5: PR Creation

1. Post completion summary to Issue:

   ```bash
   gh issue comment {url} --body "## Implementation Complete

   ### Changes from Original Plan
   [Differences if any]

   ### Final Implementation
   [What was actually implemented]"
   ```

2. Create branch, commit, and push changes
3. Create PR with Issue link:

   ```bash
   gh pr create --title "feat: ..." --body "...

   Closes owner/repo#123"
   ```

4. Post PR info to Issue:

   ```bash
   gh issue comment {url} --body "PR created: {pr_url}"
   ```

5. **STOP HERE** - Wait for user to review, approve, and merge the PR

### Phase 6: Post-Merge Completion (User-Initiated)

**Trigger**: User says "PR merged" or "complete the issue"

1. Verify PR is merged: `gh pr view {pr_number} --json state,mergedAt`
2. Update project status to "Done" (see [gh-project-api.md](references/gh-project-api.md))
3. Post final comment to Issue (optional):

   ```bash
   gh issue comment {url} --body "✅ PR merged and issue completed."
   ```

## URL Parsing

Extract from: `https://github.com/{owner}/{repo}/issues/{number}`

```text
owner = URL segment after github.com
repo = URL segment after owner
number = URL segment after "issues"
```

## Error Handling

- **No project linked**: Skip status updates, warn user
- **Permission denied**: Suggest `gh auth refresh -s project`
- **Field not found**: Search for "Status" field dynamically

## Notes

- Use conversation language for Issue comments
- PR body must include `Closes owner/repo#number` for auto-linking
- Project status auto-closes Issue when set to "Done"
- **Never update status to "Done" before PR is merged**
- Phase 6 requires explicit user confirmation to proceed

## Recommended Permission Settings

Add to `.claude/settings.local.json` to require approval for status updates:

```json
{
  "permissions": {
    "allow": [
      "Bash(gh project list:*)",
      "Bash(gh project field-list:*)"
    ]
  }
}
```

This allows project info queries but requires approval for `gh project item-edit`.
