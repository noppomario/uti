---
name: release
description: Release workflow for uti project. Use when user says "release", "create release", "bump version", "publish version", or wants to prepare a new version for distribution.
---

# Release Workflow

Create a new release for uti (GTK app + daemon).

Release is created from `main`. Ensure all features for the release are merged to `main` first.

## Steps

Open the release PR with the user's `gh` (`X.Y.Z` = new version):

```bash
git fetch origin
git switch -c release/vX.Y.Z origin/main
python .github/scripts/bump_version.py X.Y.Z
git commit -am "chore: bump version to X.Y.Z"
git push -u origin release/vX.Y.Z
gh pr create --base main --title "chore: bump version to X.Y.Z" --body "Release vX.Y.Z"
gh pr merge --auto --merge
```

When CI passes and the PR merges, `release.yml` tags the merge commit (`vX.Y.Z`, with
`GITHUB_TOKEN`) and builds and publishes in the same workflow: app RPM, daemon RPM and
GNOME extension zip as a GitHub Release.

## Version Format

`MAJOR.MINOR.PATCH` - Semantic versioning (e.g., `0.2.0`)

## Files Updated by bump_version.py

- `app/Cargo.toml`
- `app/packaging/uti.spec`
- `daemon/Cargo.toml`
- `daemon/uti-daemon.spec`
- `crates/uti-core/Cargo.toml`
- `Cargo.lock` (`cargo update --workspace`)

## Alternative: Manual Tag Push

Pushing a `vX.Y.Z` tag also runs build and release (no version bump):

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```
