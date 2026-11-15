# uti

Clipboard history, snippets and launcher popup for GNOME (Fedora 44, GNOME 50, Wayland),
toggled by double Ctrl. Three components:

- `app/` - GTK4 / libadwaita app (`uti`); pure, unit-tested UI logic in `app/src/state/`
- `daemon/` - `uti-daemon`: evdev double-Ctrl detection, uinput paste (needs `input` group)
- `gnome-extension/` - "uti for GNOME": positioning, always-on-top, paste, clipboard watch,
  panel menu (D-Bus `Shell1` service)

Packaging: `packaging/uti.spec` (COPR `noppomario/uti`), `app/packaging/` (desktop file,
metainfo, icons).

Details: [ARCHITECTURE](docs/ARCHITECTURE.md) (components, D-Bus, file locations),
[DEVELOPMENT](docs/DEVELOPMENT.md) (setup, CI, release, packaging),
[README](README.md) (usage, config files). Past decisions and their reasons: [docs/adr/](docs/adr/).

## Commands

Tools: Rust via rustup (`rust-toolchain.toml`), everything else via mise (`mise.toml`,
`mise.lock`). One-time: `mise install && just setup`.

```bash
just ci          # lint + deny + test: same checks as CI; run before committing
just fmt         # cargo fmt + biome + rumdl --fix
just test        # cargo test --workspace
just run         # app in a config sandbox (app/run-dev.sh)
just rpm         # SRPM + RPMs into dist/ (once: sudo dnf builddep packaging/uti.spec)
just bump X.Y.Z  # set the release version (used by the release skill)
```

## Rules

- English only in code, comments, docs and commit messages. Terminal output ASCII only.
- No personal information in commits or files.
- TDD: write the failing test first. Widgets are not unit-tested: put UI logic in
  `app/src/state/` (no GTK types) and test it there; `app.rs` / `ui.rs` only apply state.
- Rust doc comments on public items; no comments that restate the code.
- Recipes fail when a tool is missing (point to `mise install`); never skip checks silently.
- Docs are concise and not duplicated: one fact lives in one file.

## Git

- Trunk-based: short-lived `claude/*` branches, PRs to `main`. `Merge gate` is the only
  required check.
- Conventional Commits (`feat fix perf refactor docs test style build ci chore`), checked by
  the lefthook commit-msg hook. pre-commit formats staged files; pre-push runs
  `just lint-rust test`.
- Releases: the release skill opens `release/vX.Y.Z`; merging it tags the release and
  triggers the COPR build (`release.yml`).
