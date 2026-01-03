---
name: release
description: Release workflow for uti project. Use when user says "release", "create release", "bump version", "publish version", or wants to prepare a new version for distribution.
---

# Release Workflow

Create a new release for uti (Tauri app + daemon).

**Important**: Release is created from `develop` branch, not `main`.
Ensure all features for the release are merged to `develop` first.

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

The workflow automatically (no human intervention required):

1. Creates release branch and bumps version in all 6 locations
2. Creates PR with auto-merge enabled
3. Waits for CI to pass
4. Auto-merges PR when CI succeeds
5. Creates tag on merge (triggers release build)
6. Builds packages (Tauri RPM, daemon RPM, GNOME extension)
7. Creates GitHub Release with artifacts

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
