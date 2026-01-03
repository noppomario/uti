---
name: issue-create
description: Quick GitHub Issue creation workflow. Use when user says "create issue", "register issue", "bug report", "TODO registration", or "/issue-create {repo}". Requires repository as argument (e.g., owner/repo).
---

# Issue Creation Workflow

Create a GitHub Issue with appropriate labels.

## Usage

```text
/issue-create owner/repo
```

## Arguments

- **Repository** (required): Target repository in `owner/repo` format

ARGUMENTS: Repository name passed from skill invocation

## Workflow

1. Parse repository from ARGUMENTS
2. Gather issue details from user input
3. If unclear, ask for clarification (title, description, category)
4. Present label options and let user select
5. Fetch issue template from repository based on label (see Template Mapping)
6. Create issue body following template structure and language
7. Create issue via `gh issue create`
8. Report created issue URL

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

## Notes

- Keep it simple: This is for quick TODO registration
- Detailed planning happens in `issue-workflow` skill
- Fetch and follow repository's issue template structure and language
