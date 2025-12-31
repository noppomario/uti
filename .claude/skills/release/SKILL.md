---
name: release
description: Release workflow for uti project. Use when user says "release", "create release", "bump version", "publish version", or wants to prepare a new version for distribution.
---

# Release Workflow

Create a new release for uti (Tauri app + daemon).

## Workflow

### 1. Bump Version

Run the version bump script:

```bash
python .claude/skills/release/scripts/bump_version.py <version>
```

Example:

```bash
python .claude/skills/release/scripts/bump_version.py 0.2.0
```

This updates all 6 version locations:

- `package.json` (root)
- `app/package.json`
- `app/src-tauri/tauri.conf.json`
- `app/src-tauri/Cargo.toml`
- `daemon/Cargo.toml`
- `daemon/double-ctrl.spec`

### 2. Run Tests

```bash
bun run ci:local
```

All checks must pass before proceeding.

### 3. Commit Changes

```bash
git add -A
git commit -m "chore: bump version to X.Y.Z"
```

### 4. Create and Push Tag

```bash
git tag vX.Y.Z
git push origin main --tags
```

GitHub Actions will automatically:

- Build daemon and app RPMs
- Create a GitHub Release
- Upload RPM packages as release assets

## Version Format

Use semantic versioning: `MAJOR.MINOR.PATCH`

- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

## Scripts

### bump_version.py

Updates version in all 6 locations simultaneously. Validates version format
before making changes.
