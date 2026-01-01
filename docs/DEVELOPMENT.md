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

## Architecture

For architecture documentation with diagrams, see [ARCHITECTURE.md](ARCHITECTURE.md).

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

### Initial Setup (one-time)

Add your user to `input` group to access keyboard devices:

```bash
sudo usermod -a -G input $USER
```

**Important**: Logout and login again for group membership to take effect.

Verify group membership:

```bash
groups
# Should include "input"
```

### Running the Application

```bash
# Terminal 1: Run daemon (no sudo needed after group setup)
cd daemon
cargo run

# Terminal 2: Run Tauri app
cd app
bun run tauri:dev
```

### Why input group?

- Daemon needs to read `/dev/input/event*` (keyboard devices)
- These devices are owned by `root:input` with `660` permissions
- Adding user to `input` group grants read access
- This is the standard Linux approach (similar to `docker`, `audio` groups)
- **Reversible**: Remove with `sudo gpasswd -d $USER input`
- **Required for both development and production** - The daemon runs as a user
  session service, not as root

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

Daemon needs access to `/dev/input/*` devices.

**Solution**: Add user to `input` group (see Development Workflow section above):

```bash
sudo usermod -aG input $USER
# Log out and log back in, then:
cargo run  # No sudo needed
```

### Slow first build

First Rust build downloads and compiles all dependencies (~5 minutes).
Subsequent builds are much faster (<30 seconds for small changes).

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

## Release Process

### Version Update

Update version in all 6 locations:

| File | Format |
| ---- | ------ |
| `package.json` (root) | `"version": "X.Y.Z"` |
| `app/package.json` | `"version": "X.Y.Z"` |
| `app/src-tauri/tauri.conf.json` | `"version": "X.Y.Z"` |
| `app/src-tauri/Cargo.toml` | `version = "X.Y.Z"` |
| `daemon/Cargo.toml` | `version = "X.Y.Z"` |
| `daemon/uti-daemon.spec` | `Version:        X.Y.Z` |

**Note**: All components share the same version number for unified releases.

### Creating a Release

1. Ensure all tests pass:

   ```bash
   bun run ci:local
   ```

2. Commit version changes:

   ```bash
   git add -A
   git commit -m "chore: bump version to X.Y.Z"
   ```

3. Create and push a version tag:

   ```bash
   git tag vX.Y.Z
   git push origin main --tags
   ```

4. GitHub Actions will automatically:
   - Build daemon and app RPMs
   - Create a GitHub Release
   - Upload RPM packages as release assets

### Manual Release (workflow_dispatch)

For testing or manual releases:

1. Go to GitHub Actions
2. Select "Release" workflow
3. Click "Run workflow"
4. Enter version number (e.g., `0.2.0`)

### Release Artifacts

Each release includes:

- `uti-X.Y.Z-1.x86_64.rpm` - Tauri GUI application
- `uti-daemon-X.Y.Z-1.x86_64.rpm` - Keyboard daemon
- `gnome-extension.zip` - GNOME Shell extension (uti for GNOME)

---

## Additional Resources

- [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Project Memory](../CLAUDE.md)
- [Code Conventions](../.claude/rules/conventions.md)
- [Architectural Decisions](../.claude/rules/decisions.md)
