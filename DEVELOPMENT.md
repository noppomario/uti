# Development Environment Setup

Complete setup guide for developing **uti** on Fedora 43.

---

## Prerequisites

- **OS**: Fedora 43 (GNOME/Wayland)
- **Rust**: Install via [mise](https://mise.jdx.dev/) or [rustup](https://rustup.rs/)
- **Bun**: Install via [mise](https://mise.jdx.dev/) or [bun.sh](https://bun.sh/)
- **Rust components**: `rustfmt`, `clippy`

---

## System Dependencies

Install required development libraries for Tauri 2, D-Bus, and evdev:

```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  pango-devel \
  atk-devel \
  gdk-pixbuf2-devel \
  openssl-devel \
  dbus-devel \
  libevdev-devel \
  gcc \
  gcc-c++ \
  make \
  pkg-config \
  rpm-build \
  libxdo-devel
```

**What these provide:**
- **webkit2gtk4.1-devel**: Web rendering for Tauri UI
- **gtk3-devel, pango-devel, atk-devel, gdk-pixbuf2-devel**: GTK3 stack
- **libappindicator-gtk3-devel**: System tray support
- **librsvg2-devel**: SVG rendering
- **dbus-devel**: D-Bus IPC communication
- **libevdev-devel**: Keyboard device monitoring
- **openssl-devel**: Cryptography
- **gcc, gcc-c++, make, pkg-config**: Build toolchain
- **rpm-build**: RPM packaging
- **libxdo-devel**: Window automation

---

## Project Setup

```bash
# Clone repository
git clone https://github.com/noppomario/uti.git
cd uti

# Install dependencies
bun install
cd app && bun install && cd ..

# Build (first build takes ~5 minutes)
cargo build --release
```

---

## Verification

```bash
# Run full CI checks locally
bun run ci:local

# Expected output:
# ✓ Lint checks passed
# ✓ Type checks passed
# ✓ Build completed
```

---

## Development Workflow

```bash
# Run frontend + Tauri app
cd app
bun run tauri:dev

# Run daemon (requires root, in separate terminal)
cd daemon
sudo cargo run
```

---

## Troubleshooting

### "webkit2gtk-4.1 not found"
```bash
sudo dnf install webkit2gtk4.1-devel
```

### "libevdev not found"
```bash
sudo dnf install libevdev-devel
```

### "permission denied" when running daemon
Daemon requires root to access `/dev/input`:
```bash
sudo cargo run
```

### Slow first build
First Rust build downloads and compiles all dependencies (~5 minutes). Subsequent builds are much faster (<30 seconds for small changes).

---

## IDE Setup (VSCode)

Recommended extensions (listed in `.vscode/extensions.json`):
- **Biome**: TypeScript/React linting and formatting
- **rust-analyzer**: Rust language support
- **Tauri**: Tauri development tools

Settings (`.vscode/settings.json`) are pre-configured:
- Format on save: Enabled
- Biome for TypeScript/React
- rust-analyzer for Rust

---

## Additional Resources

- [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Project Memory](./CLAUDE.md)
- [Code Conventions](./.claude/rules/conventions.md)
- [Architectural Decisions](./.claude/rules/decisions.md)

---

**Last Updated**: 2025-12-27
