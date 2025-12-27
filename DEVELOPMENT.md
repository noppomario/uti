# Development Environment Setup

This document describes the complete development environment setup for the **uti** project.

## Quick Setup (Automated)

For Fedora 43 or Ubuntu 22.04+, you can use the automated setup script:

```bash
# Clone the repository
git clone https://github.com/noppomario/uti.git
cd uti

# Run setup script (installs all dependencies and builds the project)
./scripts/setup-dev.sh

# Verify setup
bun run ci:local
```

**What the script does:**
1. Installs Rust toolchain (if not already installed)
2. Installs Bun (if not already installed)
3. Installs system dependencies (Tauri, D-Bus, evdev, etc.)
4. Installs project dependencies
5. Builds the project

For manual setup or other distributions, continue reading below.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation Steps](#installation-steps)
  - [1. Rust Development Tools](#1-rust-development-tools)
  - [2. Node.js/Bun](#2-nodejsbun)
  - [3. System Dependencies](#3-system-dependencies)
  - [4. Project Dependencies](#4-project-dependencies)
- [Verification](#verification)
- [IDE Setup](#ide-setup)
- [Troubleshooting](#troubleshooting)

---

## System Requirements

- **OS**: Fedora 43 (GNOME/Wayland) - Primary target
- **Alternative**: Ubuntu 22.04+ or other Linux distributions
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 5GB free space (for dependencies and build artifacts)

---

## Installation Steps

### 1. Rust Development Tools

#### Install Rust via rustup

```bash
# Install rustup (Rust toolchain installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, select default installation (option 1)
# Restart your shell or run:
source $HOME/.cargo/env

# Verify installation
rustc --version   # Should show 1.75+ or later
cargo --version
```

#### Install Rust components

```bash
# Required for code formatting and linting
rustup component add rustfmt clippy
```

---

### 2. Node.js/Bun

**Option A: Install Bun (Recommended)**

```bash
# Install Bun (fast JavaScript runtime)
curl -fsSL https://bun.sh/install | bash

# Restart shell or run:
source $HOME/.bashrc  # or ~/.zshrc

# Verify
bun --version  # Should show 1.0+
```

**Option B: Use npm/Node.js**

If you prefer npm:
```bash
# Fedora
sudo dnf install nodejs npm

# Ubuntu
sudo apt install nodejs npm

# Verify
node --version  # Should show 18+
npm --version
```

**Note**: Project uses Bun by default, but npm works as well.

---

### 3. System Dependencies

#### Fedora 43 (Primary Target)

```bash
# Update package database
sudo dnf update

# Install Tauri development dependencies
sudo dnf install -y \
  webkit2gtk4.1-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  pango-devel \
  atk-devel \
  gdk-pixbuf2-devel \
  openssl-devel \
  curl \
  wget \
  file

# Install D-Bus development libraries
sudo dnf install -y dbus-devel

# Install evdev development libraries (for daemon)
sudo dnf install -y libevdev-devel

# Install build tools
sudo dnf install -y \
  gcc \
  gcc-c++ \
  make \
  pkg-config \
  rpm-build

# Install xdotool (optional, for window automation)
sudo dnf install -y libxdo-devel
```

#### Ubuntu/Debian (Reference)

```bash
# Update package database
sudo apt-get update

# Install Tauri development dependencies
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libgtk-3-dev \
  libgdk-pixbuf-2.0-dev \
  libpango1.0-dev \
  libatk1.0-dev \
  libsoup-3.0-dev

# Install D-Bus development libraries
sudo apt-get install -y libdbus-1-dev

# Install evdev development libraries
sudo apt-get install -y libevdev-dev

# Install pkg-config
sudo apt-get install -y pkg-config
```

---

### 4. Project Dependencies

#### Clone and setup

```bash
# Clone the repository
git clone https://github.com/noppomario/uti.git
cd uti

# Install root dependencies (monorepo task runner)
bun install  # or: npm install

# Install frontend dependencies
cd app
bun install  # or: npm install
cd ..

# Build Rust dependencies (first build takes ~5 minutes)
cargo build --release
```

---

## Verification

Run the following commands to verify your setup:

```bash
# Check Rust toolchain
rustc --version
cargo --version
rustfmt --version
cargo clippy --version

# Check Node.js/Bun
bun --version  # or: node --version && npm --version

# Check system libraries (should not show "not found")
pkg-config --modversion webkit2gtk-4.1
pkg-config --modversion dbus-1
pkg-config --modversion libevdev

# Run full CI checks locally
bun run ci:local

# If successful, you should see:
# ✓ Lint checks passed
# ✓ Type checks passed
# ✓ Build completed
```

---

## IDE Setup

### VSCode (Recommended)

Install recommended extensions:

```bash
# Extensions are listed in .vscode/extensions.json
# VSCode will prompt to install them automatically

# Or install manually:
code --install-extension biomejs.biome
code --install-extension rust-lang.rust-analyzer
code --install-extension tauri-apps.tauri-vscode
```

**Settings**: `.vscode/settings.json` is already configured:
- Format on save: Enabled
- Biome for TypeScript/React
- rust-analyzer for Rust

---

## Troubleshooting

### Common Issues

#### 1. "webkit2gtk-4.1 not found"

**Fedora**:
```bash
sudo dnf install webkit2gtk4.1-devel
```

**Ubuntu** (if using older version):
```bash
# Try webkit2gtk-4.0 instead
sudo apt-get install libwebkit2gtk-4.0-dev
```

#### 2. "pkg-config not found"

```bash
# Fedora
sudo dnf install pkg-config

# Ubuntu
sudo apt-get install pkg-config
```

#### 3. "libevdev not found"

```bash
# Fedora
sudo dnf install libevdev-devel

# Ubuntu
sudo apt-get install libevdev-dev
```

#### 4. Cargo build fails with linker errors

Install C/C++ compilers:
```bash
# Fedora
sudo dnf groupinstall "Development Tools"

# Ubuntu
sudo apt-get install build-essential
```

#### 5. Permission denied when running daemon

The daemon requires root privileges to access `/dev/input`:
```bash
sudo cargo run --bin double-ctrl
# or
sudo ./target/release/double-ctrl
```

#### 6. Bun not found after installation

Restart your terminal or run:
```bash
source $HOME/.bashrc  # or ~/.zshrc
```

---

## Quick Start After Setup

```bash
# Run development server (frontend + Tauri)
cd app
bun run tauri:dev

# In another terminal, run daemon (requires root)
cd daemon
sudo cargo run

# Run quality checks before committing
bun run ci:local
```

---

## Additional Resources

- [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Bun Documentation](https://bun.sh/docs)
- [Project Memory](./CLAUDE.md)
- [Conventions](./.claude/rules/conventions.md)
- [Architectural Decisions](./.claude/rules/decisions.md)

---

**Last Updated**: 2025-12-27
