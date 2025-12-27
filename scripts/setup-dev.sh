#!/bin/bash
# Development Environment Setup Script for uti project
# Target: Fedora 43 (primary), Ubuntu 22.04+ (secondary)

set -e  # Exit on error

echo "===================================================================="
echo "  uti - Development Environment Setup"
echo "===================================================================="
echo ""

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
    OS_VERSION=$VERSION_ID
else
    echo "❌ Cannot detect OS. Please install dependencies manually."
    echo "   See DEVELOPMENT.md for instructions."
    exit 1
fi

echo "Detected OS: $OS $OS_VERSION"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# ============================================
# 1. Install Rust
# ============================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Checking Rust installation..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if command_exists rustc; then
    RUST_VERSION=$(rustc --version)
    echo "✓ Rust is already installed: $RUST_VERSION"
else
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✓ Rust installed successfully"
fi

# Install Rust components
echo "Installing rustfmt and clippy..."
rustup component add rustfmt clippy
echo "✓ Rust components installed"
echo ""

# ============================================
# 2. Install Bun
# ============================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. Checking Bun installation..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if command_exists bun; then
    BUN_VERSION=$(bun --version)
    echo "✓ Bun is already installed: v$BUN_VERSION"
else
    echo "Installing Bun..."
    curl -fsSL https://bun.sh/install | bash
    export PATH="$HOME/.bun/bin:$PATH"
    echo "✓ Bun installed successfully"
    echo "⚠️  You may need to restart your shell or run: source ~/.bashrc"
fi
echo ""

# ============================================
# 3. Install System Dependencies
# ============================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. Installing system dependencies..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$OS" == "fedora" ]; then
    echo "Installing Fedora dependencies..."
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
        file \
        dbus-devel \
        libevdev-devel \
        gcc \
        gcc-c++ \
        make \
        pkg-config \
        rpm-build \
        libxdo-devel
    echo "✓ Fedora dependencies installed"

elif [ "$OS" == "ubuntu" ] || [ "$OS" == "debian" ]; then
    echo "Installing Ubuntu/Debian dependencies..."
    sudo apt-get update
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
        libsoup-3.0-dev \
        libdbus-1-dev \
        libevdev-dev \
        pkg-config
    echo "✓ Ubuntu/Debian dependencies installed"

else
    echo "⚠️  Unsupported OS: $OS"
    echo "   Please install dependencies manually."
    echo "   See DEVELOPMENT.md for the list of required packages."
    exit 1
fi
echo ""

# ============================================
# 4. Install Project Dependencies
# ============================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4. Installing project dependencies..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Install root dependencies
echo "Installing root dependencies (monorepo task runner)..."
bun install || npm install

# Install app dependencies
echo "Installing app dependencies..."
cd app
bun install || npm install
cd ..

echo "✓ Project dependencies installed"
echo ""

# ============================================
# 5. Build Project
# ============================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5. Building project (this may take 5+ minutes on first run)..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "Building Rust workspace..."
cargo build --release
echo "✓ Build completed"
echo ""

# ============================================
# Summary
# ============================================
echo "===================================================================="
echo "  ✓ Setup Complete!"
echo "===================================================================="
echo ""
echo "Installed:"
echo "  - Rust:   $(rustc --version)"
echo "  - Bun:    $(bun --version 2>/dev/null || echo 'N/A (restart shell)')"
echo "  - Cargo:  $(cargo --version)"
echo ""
echo "Next steps:"
echo "  1. Verify setup: bun run ci:local"
echo "  2. Start development: cd app && bun run tauri:dev"
echo "  3. Run daemon: cd daemon && sudo cargo run"
echo ""
echo "For more information, see:"
echo "  - DEVELOPMENT.md (detailed setup guide)"
echo "  - CLAUDE.md (quick reference)"
echo "  - README.md (project overview)"
echo ""
echo "Happy coding! 🚀"
