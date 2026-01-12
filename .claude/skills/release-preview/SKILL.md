---
name: release-preview
description: Preview changes for the next release. Triggers on "release preview", "changelog preview", "what's in next release", or Japanese equivalents like "リリース内容確認", "次のリリース".
---

# Release Preview

Display unreleased changes since the last version tag.

## Workflow

1. Get latest version tag: `git tag --sort=-version:refname | head -1`
2. List commits since tag: `git log --oneline <tag>..HEAD --no-merges`
3. Generate changelog: `bunx git-cliff --unreleased --strip header`
4. Summarize: feature count, fix count, overall assessment

## Output Format

Report in Japanese with:

- Categorized change list (Features, Fixes, Refactor, etc.)
- Commit hash and description for each change
- Summary assessment (user-facing vs developer-focused changes)
- Prompt to run `/release` skill if user wants to proceed
