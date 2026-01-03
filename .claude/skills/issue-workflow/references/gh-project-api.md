# GitHub Project API Reference

Commands for managing GitHub Projects via `gh` CLI.

## Get Project Item ID (GraphQL)

The `gh issue view --json projectItems` does NOT return item_id or project_id.
Use GraphQL API instead:

```bash
gh api graphql -f query='
{
  repository(owner: "{owner}", name: "{repo}") {
    issue(number: {issue_number}) {
      projectItems(first: 1) {
        nodes {
          id
          project {
            id
            number
          }
        }
      }
    }
  }
}'
```

Response structure:

```json
{
  "data": {
    "repository": {
      "issue": {
        "projectItems": {
          "nodes": [
            {
              "id": "PVTI_...",
              "project": {
                "id": "PVT_...",
                "number": 1
              }
            }
          ]
        }
      }
    }
  }
}
```

Extract:

- `item_id`: data.repository.issue.projectItems.nodes[0].id
- `project_id`: data.repository.issue.projectItems.nodes[0].project.id
- `project_number`: data.repository.issue.projectItems.nodes[0].project.number

## Get Project Fields

```bash
gh project field-list {project_number} --owner {owner} --format json
```

Response structure:

```json
{
  "fields": [
    {
      "id": "PVTSSF_...",
      "name": "Status",
      "type": "ProjectV2SingleSelectField",
      "options": [
        { "id": "f75ad846", "name": "Todo" },
        { "id": "47fc9ee4", "name": "In Progress" },
        { "id": "98236657", "name": "Done" }
      ]
    }
  ]
}
```

Extract:

- `status_field_id`: Find field where name == "Status"
- `in_progress_option_id`: Find option where name == "In Progress"
- `done_option_id`: Find option where name == "Done"

## Update Project Item Status

```bash
gh project item-edit \
  --id {item_id} \
  --field-id {status_field_id} \
  --project-id {project_id} \
  --single-select-option-id {option_id}
```

## Complete Example: Set Status to "In Progress"

### Step 1: Get project item ID via GraphQL

```bash
gh api graphql -f query='
{
  repository(owner: "{owner}", name: "{repo}") {
    issue(number: {issue_number}) {
      projectItems(first: 1) {
        nodes {
          id
          project { id number }
        }
      }
    }
  }
}'
```

Extract from response:

- `ITEM_ID`: `.data.repository.issue.projectItems.nodes[0].id`
- `PROJECT_ID`: `.data.repository.issue.projectItems.nodes[0].project.id`
- `PROJECT_NUM`: `.data.repository.issue.projectItems.nodes[0].project.number`

### Step 2: Get field info

```bash
gh project field-list {PROJECT_NUM} --owner {owner} --format json \
  | jq '.fields[] | select(.name=="Status")'
```

Extract:

- `STATUS_FIELD_ID`: `.id`
- `IN_PROGRESS_ID`: `.options[] | select(.name=="In Progress") | .id`
- `DONE_ID`: `.options[] | select(.name=="Done") | .id`

### Step 3: Update status

```bash
gh project item-edit \
  --id {ITEM_ID} \
  --field-id {STATUS_FIELD_ID} \
  --project-id {PROJECT_ID} \
  --single-select-option-id {OPTION_ID}
```

## Permission Requirements

If you get permission errors:

```bash
gh auth refresh -s project
```

## Notes

- Project IDs start with `PVT_`
- Item IDs start with `PVTI_`
- Field IDs start with `PVTSSF_` (single select) or `PVTF_` (other types)
- Option IDs are short hex strings (e.g., `f75ad846`)
