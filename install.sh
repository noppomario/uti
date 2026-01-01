#!/bin/bash
# uti installer - Downloads and installs uti and uti-daemon from GitHub Releases
set -e

REPO="noppomario/uti"
GITHUB_API="https://api.github.com/repos/${REPO}/releases/latest"

echo "=== uti Installer ==="
echo ""

# Check prerequisites
check_prerequisites() {
    local missing=()

    if ! command -v curl &>/dev/null; then
        missing+=("curl")
    fi

    if ! command -v dnf &>/dev/null; then
        missing+=("dnf")
    fi

    if ! command -v systemctl &>/dev/null; then
        missing+=("systemd")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        echo "Error: Missing required tools: ${missing[*]}"
        echo ""
        echo "Please install them first:"
        echo "  sudo dnf install ${missing[*]}"
        exit 1
    fi
}

# Get the latest release version from GitHub
get_latest_version() {
    curl -s "$GITHUB_API" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/'
}

# Download a file from URL to a temporary location
download_file() {
    local url="$1"
    local filename="$2"
    local dest="/tmp/${filename}"

    echo "  Downloading ${filename}..." >&2
    curl -L -o "$dest" "$url" 2>/dev/null
    echo "$dest"
}

# Main installation logic
main() {
    check_prerequisites

    echo "[1/5] Fetching latest release information..."
    local version
    version=$(get_latest_version)

    if [ -z "$version" ]; then
        echo "Error: Could not fetch latest version from GitHub"
        echo "Please check your internet connection or try again later."
        exit 1
    fi

    echo "  Latest version: v${version}"
    echo ""

    # Construct download URLs
    local base_url="https://github.com/${REPO}/releases/download/v${version}"
    local daemon_rpm="uti-daemon-${version}-1.x86_64.rpm"
    local uti_rpm="uti-${version}-1.x86_64.rpm"

    echo "[2/5] Downloading packages..."
    local daemon_path uti_path
    daemon_path=$(download_file "${base_url}/${daemon_rpm}" "$daemon_rpm")
    uti_path=$(download_file "${base_url}/${uti_rpm}" "$uti_rpm")
    echo ""

    echo "[3/5] Installing packages..."
    echo "  This will require sudo access."
    echo ""
    sudo dnf install -y "$daemon_path" "$uti_path"
    echo ""

    # Clean up downloaded files
    rm -f "$daemon_path" "$uti_path"

    echo "[4/5] Checking input group membership..."
    if ! groups | grep -q "\binput\b"; then
        echo ""
        echo "  ⚠ You are not in the 'input' group."
        echo "  The daemon needs this to monitor keyboard events."
        echo ""
        echo "  Adding you to the input group..."
        sudo usermod -aG input "$USER"
        echo ""
        echo "  ✓ Added to input group."
        echo "  ⚠ You must log out and log back in for this to take effect!"
        echo ""
    else
        echo "  ✓ Already in input group."
    fi

    echo "[5/6] Enabling systemd user service..."
    systemctl --user daemon-reload
    systemctl --user enable --now uti-daemon.service

    # ANSI color codes
    local RED='\033[0;31m'
    local GREEN='\033[0;32m'
    local YELLOW='\033[1;33m'
    local NC='\033[0m' # No Color
    local BOLD='\033[1m'

    echo ""
    echo "[6/6] Installing uti for GNOME (if applicable)..."
    local gnome_installed=false
    if [ "$XDG_CURRENT_DESKTOP" = "GNOME" ] || command -v gnome-extensions &>/dev/null; then
        local ext_uuid="uti@noppomario.github.io"
        local ext_dir="$HOME/.local/share/gnome-shell/extensions/${ext_uuid}"
        local ext_url="${base_url}/gnome-extension.zip"

        echo "  Downloading uti for GNOME..."
        local ext_zip="/tmp/uti-gnome-extension.zip"
        if curl -L -o "$ext_zip" "$ext_url" 2>/dev/null; then
            mkdir -p "$ext_dir"
            unzip -o -q "$ext_zip" -d "$ext_dir"
            rm -f "$ext_zip"
            # Compile schemas if glib-compile-schemas is available
            if command -v glib-compile-schemas &>/dev/null && [ -d "$ext_dir/schemas" ]; then
                glib-compile-schemas "$ext_dir/schemas" 2>/dev/null || true
            fi
            echo "  ✓ Extension installed to $ext_dir"
            gnome_installed=true
        else
            echo "  ⚠ Could not download extension (optional, skipping)"
        fi
    else
        echo "  Skipping (not a GNOME environment)"
    fi

    echo ""
    echo "==========================================="
    echo -e " ${GREEN}✓ Installation complete!${NC}"
    echo "==========================================="
    echo ""
    echo -e "${RED}${BOLD}╔═════════════════════════╗${NC}"
    echo -e "${RED}${BOLD}║  ⚠ YOU MUST LOG OUT AND LOG BACK IN TO USE UTI  ║${NC}"
    echo -e "${RED}${BOLD}╚═════════════════════════╝${NC}"
    echo ""
    echo "After logging back in:"
    echo "  1. Run 'uti' from the command line or application menu"
    echo ""

    # GNOME-specific instructions
    if [ "$gnome_installed" = true ]; then
        echo -e "${YELLOW}GNOME Users:${NC}"
        echo "  After logging back in, enable the extension with:"
        echo "    gnome-extensions enable uti@noppomario.github.io"
        echo ""
    fi

    echo "Service status:"
    systemctl --user status uti-daemon.service --no-pager || true
    echo ""
    echo "To uninstall:"
    echo "  sudo dnf remove uti uti-daemon"
    echo ""
}

main "$@"
