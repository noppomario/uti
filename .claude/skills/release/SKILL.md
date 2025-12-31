---
name: release
description: Release workflow for uti project. Use when user says "release", "create release", "bump version", "publish version", or wants to prepare a new version for distribution.
---

# Release Workflow

Create a new release for uti (Tauri app + daemon).

## Workflow

### 1. Create Release Branch

```bash
git checkout main && git pull origin main
git checkout -b claude/release-X.Y.Z
```

### 2. Bump Version

```bash
python .claude/skills/release/scripts/bump_version.py <version>
```

Updates all 6 version locations:

- `package.json` (root)
- `app/package.json`
- `app/src-tauri/tauri.conf.json`
- `app/src-tauri/Cargo.toml`
- `daemon/Cargo.toml`
- `daemon/double-ctrl.spec`

### 3. Commit and Push

```bash
git add -A
git commit -m "chore: bump version to X.Y.Z"
git push -u origin claude/release-X.Y.Z
```

### 4. Create PR

```bash
gh pr create --title "chore: bump version to X.Y.Z" --body "Release X.Y.Z"
```

Wait for CI checks to pass, then merge the PR.

### 5. Create and Push Tag (after PR merge)

```bash
git checkout main && git pull origin main
git tag vX.Y.Z
git push origin vX.Y.Z
```

GitHub Actions will automatically build RPMs and create a GitHub Release.

## Version Format

`MAJOR.MINOR.PATCH` - Semantic versioning (e.g., `0.2.0`)

## Scripts

### bump_version.py

Updates version in all 6 locations. Validates semver format (`X.Y.Z`).
