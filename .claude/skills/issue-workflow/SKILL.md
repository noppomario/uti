---
name: issue-workflow
description: GitHub Issue-based task execution workflow with project management. Use when user says "work on issue", "implement issue", "start issue", or "/issue-workflow {issue-url}". Includes project status management, plan mode investigation, and PR auto-linking.
---

# Issue Workflow

Execute development tasks based on GitHub Issues with full project management.

## Usage

```text
/issue-workflow https://github.com/owner/repo/issues/123
```

**IMPORTANT**: Execute this skill in NORMAL mode (not Plan mode). The skill will
use EnterPlanMode tool to transition to Plan mode after preliminary actions.

## Key Instruction

1. **Execute PRELIMINARY ACTIONS first** (in normal mode, with tool execution)
2. **Call EnterPlanMode** to enter Claude Code's official Plan mode
3. **Create plan following the WORKFLOW TEMPLATE** (in Plan mode)
4. **Execute POST-APPROVAL ACTIONS** after user approves the plan

## Arguments

- **Issue URL** (required): Full GitHub issue URL

ARGUMENTS: Issue URL passed from skill invocation

## PRELIMINARY ACTIONS

Execute these steps in NORMAL mode (before entering Plan mode):

```text
1. Parse Issue URL → extract `owner`, `repo`, `issue_number`
2. Fetch issue details: `gh issue view {url} --json title,body,labels,projectItems`
3. Read linked issues mentioned in comments
4. Update project status to "In Progress" (see references/gh-project-api.md)
5. Call EnterPlanMode tool to enter Plan mode
```

After completing preliminary actions and entering Plan mode, create plan using
the WORKFLOW TEMPLATE below.

## WORKFLOW TEMPLATE

Create your plan with EXACTLY these phases. Each phase must be included.

---

### Phase 1: Investigation and Implementation Plan

**Goal**: Create detailed technical plan

- [ ] Investigate codebase based on issue requirements
- [ ] Identify files to modify/create
- [ ] Design implementation approach
- [ ] List test items

**Plan output**: Technical implementation steps

---

### --- POST-APPROVAL ACTIONS (after user approves plan) ---

**This is NOT a phase** - execute immediately after user approves the plan:

- [ ] Post **FULL plan content** to Issue (see Post Plan to Issue section)

Then proceed to Phase 2.

---

### Phase 2: Implementation

**Goal**: Execute the technical plan

- [ ] Implement changes following Phase 1 plan
- [ ] Run tests: `bun run ci:local`
- [ ] Track deviations from original plan

---

### Phase 3: Documentation Review

**Goal**: Ensure documentation is updated

Check these files for potential updates:

- [ ] README.md - User-facing documentation
- [ ] CLAUDE.md - Project memory
- [ ] .claude/rules/*.md - Architecture decisions

Report findings to user and update as specified.

---

### Phase 4: Completion Summary

**Goal**: Document implementation and wait for feedback

- [ ] Post completion summary to Issue (see Post Summary to Issue section)

**CHECKPOINT**: Wait for user testing and feedback.

---

### Phase 5: Feedback

**Goal**: Address user feedback from testing

- [ ] Address any feedback from user testing
- [ ] Iterate until user confirms implementation is complete

**CHECKPOINT**: Wait for user to confirm feedback is complete.

---

### Phase 6: Final Review and PR

**Goal**: Re-verify documentation, summarize, and create PR

**IMPORTANT**: After feedback is complete, re-run Phase 3 and Phase 4 before creating PR.

- [ ] Re-run Phase 3: Documentation Review (check if feedback changes require doc updates)
- [ ] Re-run Phase 4: Completion Summary (post final summary to Issue)
- [ ] Create branch, commit, push
- [ ] Create PR: `gh pr create --base develop --title "..." --body "Closes owner/repo#123"`
- [ ] Post PR link to Issue

**STOP**: Wait for user to review and merge PR.

---

### Phase 7: Post-Merge (User-Initiated)

**Trigger**: User says "PR merged" or "complete the issue"

- [ ] Verify PR is merged
- [ ] Update project status to "Done"
- [ ] Post final comment (optional)

---

## Post Plan to Issue

After user approves the plan, post the full plan content to the Issue.

**Command**:

```bash
# Read the plan file and post its COMPLETE content
# Plan file path is shown in system messages when in Plan mode (e.g., /tmp/xxx/plan.md)
gh issue comment {url} --body "$(cat <<'EOF'
## Implementation Plan

$(cat {plan_file_path})
EOF
)"
```

**CRITICAL REQUIREMENTS** (MUST follow ALL):

- **NEVER summarize** - Post the EXACT content of the plan file
- **Use `cat` command** - Read the plan file from the path shown in Plan mode system messages
- **Include everything** - Code snippets, tables, test items, phases, all sections
- **No paraphrasing** - Copy the plan content verbatim

If you summarize instead of posting the full content, you are violating this
skill's requirements.

**Language**: Use the **conversation language** for Issue comments.

- If the conversation is in Japanese → Write comments in Japanese
- If the conversation is in English → Write comments in English

## Post Summary to Issue

```bash
gh issue comment {url} --body "$(cat <<'EOF'
## Implementation Complete

### Changes from Original Plan

[Differences if any, or "None"]

### What Was Implemented

[List of implemented items]

### Verification

[Test results, CI status]
EOF
)"
```

## URL Parsing

Extract from: `https://github.com/{owner}/{repo}/issues/{number}`

## User Wait Points Summary

| Point | Trigger | Purpose |
| ----- | ------- | ------- |
| Plan approval | User approves plan in Plan mode | Confirm implementation approach |
| Testing | Phase 4 CHECKPOINT | User validates implementation |
| Feedback complete | Phase 5 CHECKPOINT | Confirm all feedback addressed |
| PR merge | Phase 6 STOP | GitHub review process |
| Post-merge | Phase 7 (user-initiated) | Final cleanup |

## Error Handling

- **No project linked**: Skip status updates, warn user
- **Permission denied**: Suggest `gh auth refresh -s project`
- **Field not found**: Search for "Status" field dynamically

## Notes

- PR body must include `Closes owner/repo#number` for auto-linking
- Project status auto-closes Issue when set to "Done"
- Never update status to "Done" before PR is merged

## Recommended Permission Settings

Add to `.claude/settings.local.json`:

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
