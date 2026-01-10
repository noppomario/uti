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

1. Post **full plan content** to Issue:

   ```bash
   # MUST read the actual plan file and post its COMPLETE content
   gh issue comment {url} --body "## Implementation Plan

   $(cat {plan_file_path})"
   ```

   **CRITICAL REQUIREMENTS** (MUST follow ALL):

   - **NEVER summarize** - Post the EXACT content of the plan file
   - **Use `cat` command** - Read the plan file path shown in system messages
   - **Include everything** - Code snippets, tables, test items, phases, sources
   - **No paraphrasing** - Copy the plan content verbatim

   If you summarize instead of posting the full content, you are violating this
   skill's requirements.

   **IMPORTANT**: Use the **conversation language** for Issue comments.
   If the conversation is in Japanese, write in Japanese.
   If the conversation is in English, write in English.

2. Update project status to "In Progress" using the commands in
   [gh-project-api.md](references/gh-project-api.md)

3. **MANDATORY CHECKPOINT - STOP AND WAIT**:

   After posting the plan to the Issue, you MUST:

   - Inform the user that the plan has been posted
   - Provide the Issue comment URL
   - Ask the user to review the posted plan
   - **STOP and wait for explicit approval** (e.g., "OK", "proceed", "continue")

   **DO NOT proceed to Phase 4 until the user explicitly confirms.**

   Example message:

   > Plan posted to Issue: {comment_url}
   >
   > Please review the posted plan. Say "OK" to proceed with implementation.

### Phase 4: Implementation (User Approval Required)

1. Follow the plan to implement changes
2. Periodically check remaining tasks
3. Track any deviations from original plan

### Phase 5: Documentation Review

Before creating PR, investigate if documentation updates are needed.

1. Check these files for potential updates:

   - **Project docs**: README.md, docs/*.md
   - **Claude memory**: CLAUDE.md, .claude/rules/*.md

   Consider:

   - Does the feature need user-facing documentation?
   - Are there architecture or setup changes?
   - Should a new ADR be recorded for significant decisions?

2. Report findings to user:

   > **Documentation Review**
   >
   > - README.md: [No changes needed / Needs update: reason]
   > - CLAUDE.md: [No changes needed / Needs update: reason]
   > - .claude/rules/decisions.md: [No ADR needed / New ADR recommended: topic]
   >
   > Reply with files to update, or "none" to skip.

3. Make documentation updates as specified by user

### Phase 6: Completion Summary

1. Post completion summary to Issue:

   ```bash
   gh issue comment {url} --body "## Implementation Complete

   ### Changes from Original Plan
   [Differences if any]

   ### Final Implementation
   [What was actually implemented]"
   ```

   **IMPORTANT**: Use the **conversation language** for Issue comments.
   If the conversation is in Japanese, write in Japanese.
   If the conversation is in English, write in English.

2. **MANDATORY CHECKPOINT - STOP AND WAIT**:

   After posting the completion summary, you MUST:

   - Inform the user that the summary has been posted
   - Provide the Issue comment URL
   - Ask the user to review the summary
   - **STOP and wait for explicit approval** before creating PR

   **DO NOT create a PR until the user explicitly confirms.**

   Example message:

   > Completion summary posted to Issue: {comment_url}
   >
   > Please review. Say "OK" to create PR.

### Phase 6.5: Additional Changes (If Any)

If changes are made after the completion summary (Phase 6):

1. Post update comment to Issue:

   ```bash
   gh issue comment {url} --body "## Additional Changes

   [Description of changes made after the completion summary]

   - Change 1
   - Change 2"
   ```

2. Wait for user confirmation before proceeding to PR

This phase is only needed when:

- User requests additional improvements after reviewing
- Bug fixes discovered during final testing
- Any modifications after the completion summary was posted

Skip this phase if no changes were made after Phase 6.

### Phase 7: PR Creation

1. Create branch, commit, and push changes
2. Create PR with Issue link (target `develop` branch):

   ```bash
   gh pr create --base develop --title "feat: ..." --body "...

   Closes owner/repo#123"
   ```

3. Post PR info to Issue:

   ```bash
   gh issue comment {url} --body "PR created: {pr_url}"
   ```

4. **STOP HERE** - Wait for user to review, approve, and merge the PR

### Phase 8: Post-Merge Completion (User-Initiated)

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
- Phase 8 requires explicit user confirmation to proceed

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
