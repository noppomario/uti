#!/bin/bash
# Build the app RPM from a release binary.
#
# Usage: ./app/packaging/build-rpm.sh [--package-only] [version]
#   --package-only  Use the existing target/release/uti (skip cargo build)
#   version         Defaults to app/Cargo.toml

set -euo pipefail

PKG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$PKG_DIR/../.." && pwd)"

PACKAGE_ONLY=false
VERSION=""
for arg in "$@"; do
    case $arg in
        --package-only) PACKAGE_ONLY=true ;;
        *) VERSION="$arg" ;;
    esac
done
if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version' "$PROJECT_ROOT/app/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

command -v rpmbuild > /dev/null || { echo "Error: rpmbuild not found (dnf install rpm-build)"; exit 1; }

BIN="$PROJECT_ROOT/target/release/uti"
if [ "$PACKAGE_ONLY" = false ]; then
    cargo build --release -p uti --manifest-path "$PROJECT_ROOT/Cargo.toml"
fi
[ -f "$BIN" ] || { echo "Error: $BIN not found"; exit 1; }

RPMBUILD_DIR="$PROJECT_ROOT/target/rpmbuild-app"
rm -rf "$RPMBUILD_DIR"
mkdir -p "$RPMBUILD_DIR"/{SPECS,SOURCES,BUILD,RPMS,SRPMS}
sed "s/^Version:.*/Version:        $VERSION/" "$PKG_DIR/uti.spec" > "$RPMBUILD_DIR/SPECS/uti.spec"
cp "$BIN" "$PKG_DIR"/io.github.noppomario.uti.{desktop,metainfo.xml} "$RPMBUILD_DIR/SOURCES/"
cp -r "$PKG_DIR/icons" "$RPMBUILD_DIR/SOURCES/"

rpmbuild -bb "$RPMBUILD_DIR/SPECS/uti.spec" \
    --define "_topdir $RPMBUILD_DIR" \
    --define "_sourcedir $RPMBUILD_DIR/SOURCES" \
    --define "debug_package %{nil}"

mkdir -p "$PROJECT_ROOT/dist"
cp "$RPMBUILD_DIR/RPMS/"*/*.rpm "$PROJECT_ROOT/dist/"
ls -la "$PROJECT_ROOT/dist/"uti-"$VERSION"*.rpm
