---
name: issue-create
description: Quick GitHub Issue creation workflow. Use when user says "create issue", "register issue", "bug report", "TODO registration", or "/issue-create {repo}". Requires repository as argument (e.g., owner/repo).
---

# Issue Creation Workflow

Create a GitHub Issue with appropriate labels.

## Usage

```text
/issue-create owner/repo
/issue-create owner/repo --from-plan
```

## Arguments

- **Repository** (required): Target repository in `owner/repo` format
- **--from-plan** (optional): Include current plan file content in issue body

ARGUMENTS: Repository name and options passed from skill invocation

## Workflow

1. Parse repository from ARGUMENTS
2. Check if `--from-plan` flag is present
3. Gather issue details from user input
4. If unclear, ask for clarification (title, description, category)
5. Present label options and let user select
6. Fetch issue template from repository based on label (see Template Mapping)
7. Create issue body following template structure and language
8. If `--from-plan`: Append plan content to issue body (see From Plan section)
9. Create issue via `gh issue create`
10. Report created issue URL

## From Plan Option

When `--from-plan` is specified:

1. Read current plan file from `~/.claude/plans/` directory
2. Append plan content to issue body under "## Investigation Results" section
3. This enables seamless handoff to `issue-workflow`

**Plan file location**: Check system message for active plan file path, or find
the most recently modified `.md` file in the plans directory.

**Issue body structure with --from-plan**:

```markdown
[Template-based content]

## Investigation Results

[Full plan content from plan file]
```

## Label Selection Guide

Select one or more labels based on issue type:

| Label              | Use Case                    |
| ------------------ | --------------------------- |
| `bug`              | Something isn't working     |
| `enhancement`      | New feature or improvement  |
| `documentation`    | Documentation changes       |
| `question`         | Questions or investigations |
| `good first issue` | Beginner-friendly tasks     |
| `help wanted`      | Extra attention needed      |

Rarely used:

- `duplicate` - Already exists
- `invalid` - Not valid
- `wontfix` - Will not be addressed

## Template Mapping

Auto-select template based on primary label:

| Label           | Template file |
| --------------- | ------------- |
| `bug`           | `bug.md`      |
| `enhancement`   | `feature.md`  |
| `documentation` | `feature.md`  |
| `question`      | `idea.md`     |
| (other/none)    | `idea.md`     |

## Fetching Template

Before creating issue, fetch the template to follow its structure:

```bash
gh api repos/{owner}/{repo}/contents/.github/ISSUE_TEMPLATE/{template}.md \
  | jq -r '.content' | base64 -d
```

## Issue Creation Command

```bash
gh issue create -R {owner}/{repo} \
  --title "Issue title" \
  --body "Issue body following template structure" \
  --label "enhancement"
```

## Language

**Match the template's language**: Write issue title and body in the same
language as the repository's issue template.

- Fetch the template first to determine its language
- If template is in Japanese → Write issue in Japanese
- If template is in English → Write issue in English

## Handoff to issue-workflow

After creating an issue with `--from-plan`, suggest:

```text
Issue created. To implement this issue:
/issue-workflow {created_issue_url}
```

## Notes

- Keep it simple: This is for quick TODO registration
- Detailed planning happens in `issue-workflow` skill
- Fetch and follow repository's issue template structure and language
- `--from-plan` bridges investigation work to formal issue tracking
