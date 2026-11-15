#!/bin/bash
# Build the source RPM from the checked-out commit (HEAD).
#
# Usage: packaging/make-srpm.sh [outdir]   (default: dist/)
#
# Version comes from `git describe`: `0.4.0` on a release tag, `0.4.0^3.gabc1234`
# three commits after it. Uncommitted changes are not included.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTDIR="$(realpath -m "${1:-$ROOT/dist}")"
# COPR clones the repository as another user.
git() { command git -c safe.directory="$ROOT" -C "$ROOT" "$@"; }

desc=$(git describe --tags --long --match 'v[0-9]*')
read -r tag count hash <<< "$(sed -E 's/^v(.*)-([0-9]+)-(g[0-9a-f]+)$/\1 \2 \3/' <<< "$desc")"
if [ "$count" = 0 ]; then version="$tag"; else version="$tag^$count.$hash"; fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
src="$work/uti-$version"
mkdir -p "$src"
git archive HEAD | tar -x -C "$src"
(cd "$src" && cargo vendor --locked --quiet vendor > /dev/null)
tar -C "$work" -czf "$work/uti-$version.tar.gz" "uti-$version"
sed "s/^Version:.*/Version:        $version/" "$src/packaging/uti.spec" > "$work/uti.spec"

mkdir -p "$OUTDIR"
rpmbuild -bs "$work/uti.spec" \
    --define "_sourcedir $work" \
    --define "_srcrpmdir $OUTDIR"
