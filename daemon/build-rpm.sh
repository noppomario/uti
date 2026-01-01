#!/bin/bash
# Build daemon RPM package locally
#
# Prerequisites:
#   - rpm-build package (sudo dnf install rpm-build)
#   - Rust toolchain (unless --package-only is used)
#
# Usage:
#   ./daemon/build-rpm.sh [--package-only] [version]
#
# Options:
#   --package-only  Skip cargo build, use existing binary in target/release
#
# If version is not specified, reads from daemon/Cargo.toml

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse arguments
PACKAGE_ONLY=false
VERSION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --package-only)
            PACKAGE_ONLY=true
            shift
            ;;
        *)
            VERSION="$1"
            shift
            ;;
    esac
done

# Get version from Cargo.toml if not specified
if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version' "$PROJECT_ROOT/daemon/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

echo "Building daemon RPM version $VERSION..."

# Check prerequisites
if ! command -v rpmbuild &> /dev/null; then
    echo "Error: rpmbuild not found. Install with: sudo dnf install rpm-build"
    exit 1
fi

# Build release binary (unless --package-only)
if [ "$PACKAGE_ONLY" = true ]; then
    echo "Package only mode (using existing binary)..."
    if [ ! -f "$PROJECT_ROOT/target/release/uti-daemon" ]; then
        echo "Error: Binary not found at target/release/uti-daemon"
        echo "Run 'cargo build --release' first or remove --package-only flag"
        exit 1
    fi
else
    echo "Building release binary..."
    cargo build --release --manifest-path "$PROJECT_ROOT/daemon/Cargo.toml"
fi

# Create RPM build directory structure
RPMBUILD_DIR="$PROJECT_ROOT/target/rpmbuild"
mkdir -p "$RPMBUILD_DIR"/{SPECS,SOURCES,BUILD,RPMS,SRPMS}

# Update spec version and copy
sed "s/^Version:.*/Version:        $VERSION/" "$PROJECT_ROOT/daemon/uti-daemon.spec" \
    > "$RPMBUILD_DIR/SPECS/uti-daemon.spec"

# Copy build files to SOURCES (rpmbuild expects them there or uses absolute paths)
cp "$PROJECT_ROOT/target/release/uti-daemon" "$RPMBUILD_DIR/SOURCES/"
cp "$PROJECT_ROOT/daemon/systemd/uti-daemon.service" "$RPMBUILD_DIR/SOURCES/"

# Build RPM with source directory override
echo "Building RPM..."
rpmbuild -bb "$RPMBUILD_DIR/SPECS/uti-daemon.spec" \
    --define "_topdir $RPMBUILD_DIR" \
    --define "_sourcedir $RPMBUILD_DIR/SOURCES" \
    --define "debug_package %{nil}"

# Copy to dist
DIST_DIR="$PROJECT_ROOT/dist"
mkdir -p "$DIST_DIR"
cp "$RPMBUILD_DIR/RPMS/"*/*.rpm "$DIST_DIR/"

echo ""
echo "RPM built successfully!"
echo "Output: $DIST_DIR/"
ls -la "$DIST_DIR/"*.rpm 2>/dev/null | grep uti-daemon || true
