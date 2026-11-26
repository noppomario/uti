# Development tasks. Tools come from `mise install`; Rust from rust-toolchain.toml.
# CI runs the same recipes, so `just ci` passing locally means CI's checks pass.

metainfo := "app/packaging/io.github.noppomario.uti.metainfo.xml"

default:
    @just --list

# Install Git hooks (replaces husky)
setup: (_need "lefthook")
    -git config --unset core.hooksPath
    lefthook install

# Format everything in place
fmt: (_need "biome") (_need "rumdl")
    cargo fmt --all
    cd gnome-extension && biome check --write .
    rumdl check --fix .

# rustfmt check and clippy (warnings are errors)
lint-rust:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings

# Markdown lint (rumdl reads .markdownlint.json)
lint-md: (_need "rumdl")
    rumdl check .

# GNOME extension lint and format check
lint-ext: (_need "biome")
    cd gnome-extension && biome ci .

# GitHub Actions workflow lint
# (shellcheck is required: actionlint silently skips run: scripts without it)
lint-gha: (_need "actionlint") (_need "shellcheck")
    actionlint

# Shell scripts shipped or run by users
lint-sh: (_need "shellcheck")
    shellcheck install.sh packaging/make-srpm.sh app/run-dev.sh tools/e2e/run.sh .claude/skills/release/wait-copr.sh

lint: lint-rust lint-md lint-ext lint-gha lint-sh

# Advisories, licenses, bans and sources of dependencies
deny: (_need "cargo-deny")
    cargo deny --log-level error check

test:
    cargo test --workspace --locked

# End-to-end checks in a headless GNOME Shell (local only; see tools/e2e/run.sh)
e2e *args:
    tools/e2e/run.sh {{args}}

# Everything CI checks
ci: lint deny test

# Set the release version in Cargo.toml and Cargo.lock and add it to the
# AppStream release list (release skill)
bump version:
    @echo '{{version}}' | grep -Eqx '[0-9]+\.[0-9]+\.[0-9]+' || { echo "usage: just bump X.Y.Z" >&2; exit 1; }
    sed -i -E '/^\[workspace\.package\]/,/^\[/ s/^version = ".*"/version = "{{version}}"/' Cargo.toml
    grep -qx 'version = "{{version}}"' Cargo.toml
    cargo update --workspace
    sed -i "/<releases>/a\\    <release version=\"{{version}}\" date=\"$(date -u +%F)\"/>" {{metainfo}}
    grep -q 'version="{{version}}"' {{metainfo}}

# Run the app against a sandbox copy of ~/.config/uti
run *args:
    ./app/run-dev.sh {{args}}

# Build the SRPM from HEAD, then the binary RPMs offline, into dist/ (as COPR does).
# Needs the build dependencies once: sudo dnf builddep packaging/uti.spec
rpm *flags:
    rm -rf dist
    packaging/make-srpm.sh dist
    rpmbuild --rebuild dist/*.src.rpm --define "_topdir $PWD/target/rpmbuild" \
        --define "_rpmdir $PWD/dist" --define "_build_name_fmt %%{NAME}-%%{VERSION}-%%{RELEASE}.%%{ARCH}.rpm" \
        {{flags}}

[private]
_need tool:
    @command -v {{tool}} >/dev/null || { echo "{{tool}} not found: run 'mise install'" >&2; exit 1; }
