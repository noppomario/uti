---
name: release
description: Release workflow for uti project. Use when user says "release", "create release", "bump version", "publish version", or wants to prepare a new version for distribution.
---

# Release Workflow

Create a new release for uti (Tauri app + daemon).

## Usage

Run the release workflow with the desired version:

```bash
gh workflow run release.yml -f version=X.Y.Z
```

Example:

```bash
gh workflow run release.yml -f version=0.0.4
```

## What It Does

The workflow automatically:

1. Bumps version in all 6 locations
2. Commits to main branch
3. Creates and pushes tag
4. Builds packages (Tauri RPM, daemon RPM, GNOME extension)
5. Creates GitHub Release with artifacts

## Version Format

`MAJOR.MINOR.PATCH` - Semantic versioning (e.g., `0.2.0`)

## Files Updated

- `package.json` (root)
- `app/package.json`
- `app/src-tauri/tauri.conf.json`
- `app/src-tauri/Cargo.toml`
- `daemon/Cargo.toml`
- `daemon/uti-daemon.spec`

## Alternative: Manual Tag Push

If you prefer manual control, you can still create a tag manually:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

This triggers only the build and release jobs (skips version bump).
