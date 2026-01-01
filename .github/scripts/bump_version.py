#!/usr/bin/env python3
"""
Bump version in all 6 locations for uti project.

Usage:
    python bump_version.py <version>
    python bump_version.py 0.2.0

Used by: .github/workflows/release.yml
"""

import json
import re
import sys
from pathlib import Path


def get_project_root() -> Path:
    """Find project root by looking for Cargo.toml in workspace."""
    current = Path.cwd()
    while current != current.parent:
        if (current / "Cargo.toml").exists() and (current / "app").exists():
            return current
        current = current.parent
    raise RuntimeError("Could not find project root")


def update_json_version(file_path: Path, version: str) -> None:
    """Update version in a JSON file."""
    with open(file_path) as f:
        data = json.load(f)
    data["version"] = version
    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    print(f"  Updated: {file_path.relative_to(get_project_root())}")


def update_cargo_version(file_path: Path, version: str) -> None:
    """Update version in a Cargo.toml file."""
    content = file_path.read_text()
    new_content = re.sub(
        r'^version = "[^"]+"',
        f'version = "{version}"',
        content,
        count=1,
        flags=re.MULTILINE,
    )
    file_path.write_text(new_content)
    print(f"  Updated: {file_path.relative_to(get_project_root())}")


def update_rpm_spec_version(file_path: Path, version: str) -> None:
    """Update version in an RPM spec file."""
    content = file_path.read_text()
    new_content = re.sub(
        r"^Version:\s+.*$",
        f"Version:        {version}",
        content,
        count=1,
        flags=re.MULTILINE,
    )
    file_path.write_text(new_content)
    print(f"  Updated: {file_path.relative_to(get_project_root())}")


def main():
    if len(sys.argv) != 2:
        print("Usage: bump_version.py <version>")
        print("Example: bump_version.py 0.2.0")
        sys.exit(1)

    version = sys.argv[1]

    # Validate version format (semver only)
    if not re.match(r"^\d+\.\d+\.\d+$", version):
        print(f"Error: Invalid version format '{version}'")
        print("Expected format: X.Y.Z (e.g., 0.2.0)")
        sys.exit(1)

    root = get_project_root()
    print(f"Bumping version to {version}...")

    # 1. Root package.json
    update_json_version(root / "package.json", version)

    # 2. app/package.json
    update_json_version(root / "app" / "package.json", version)

    # 3. app/src-tauri/tauri.conf.json
    update_json_version(root / "app" / "src-tauri" / "tauri.conf.json", version)

    # 4. app/src-tauri/Cargo.toml
    update_cargo_version(root / "app" / "src-tauri" / "Cargo.toml", version)

    # 5. daemon/Cargo.toml
    update_cargo_version(root / "daemon" / "Cargo.toml", version)

    # 6. daemon/uti-daemon.spec
    update_rpm_spec_version(root / "daemon" / "uti-daemon.spec", version)

    print(f"\nAll 6 files updated to version {version}")


if __name__ == "__main__":
    main()
