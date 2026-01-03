# GitHub Project API Reference

Commands for managing GitHub Projects via `gh` CLI.

## Get Project Info from Issue

```bash
gh issue view {issue_url} --json projectItems
```

Response structure:

```json
{
  "projectItems": [
    {
      "id": "PVTI_...",
      "project": {
        "id": "PVT_...",
        "number": 1,
        "title": "Project Name"
      },
      "status": {
        "name": "Todo",
        "optionId": "f75ad846"
      }
    }
  ]
}
```

Extract:

- `item_id`: projectItems[0].id
- `project_id`: projectItems[0].project.id
- `project_number`: projectItems[0].project.number

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

```bash
# 1. Get project info from issue
PROJECT_INFO=$(gh issue view {issue_url} --json projectItems)
ITEM_ID=$(echo $PROJECT_INFO | jq -r '.projectItems[0].id')
PROJECT_ID=$(echo $PROJECT_INFO | jq -r '.projectItems[0].project.id')
PROJECT_NUM=$(echo $PROJECT_INFO | jq -r '.projectItems[0].project.number')

# 2. Get field info
FIELDS=$(gh project field-list $PROJECT_NUM --owner {owner} --format json)
STATUS_FIELD_ID=$(echo $FIELDS | jq -r '.fields[] | select(.name=="Status") | .id')
IN_PROGRESS_ID=$(echo $FIELDS | jq -r '.fields[] | select(.name=="Status") | .options[] | select(.name=="In Progress") | .id')

# 3. Update status
gh project item-edit \
  --id $ITEM_ID \
  --field-id $STATUS_FIELD_ID \
  --project-id $PROJECT_ID \
  --single-select-option-id $IN_PROGRESS_ID
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
