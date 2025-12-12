#!/bin/sh
# Forge Installation Script
# Usage: curl -fsSL https://get.forge.dev/install.sh | sh
#
# Environment variables:
#   FORGE_VERSION     - Version to install (default: latest)
#   FORGE_INSTALL_DIR - Installation directory (default: ~/.forge/bin)

set -e

# Configuration
FORGE_VERSION="${FORGE_VERSION:-latest}"
INSTALL_DIR="${FORGE_INSTALL_DIR:-$HOME/.forge/bin}"
GITHUB_REPO="${FORGE_GITHUB_REPO:-LayerDynamics/forge}"

# Colors (if terminal supports them)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

info() {
    printf "${BLUE}info${NC}: %s\n" "$1"
}

success() {
    printf "${GREEN}success${NC}: %s\n" "$1"
}

warn() {
    printf "${YELLOW}warning${NC}: %s\n" "$1"
}

error() {
    printf "${RED}error${NC}: %s\n" "$1" >&2
    exit 1
}

# Detect platform
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac

    case "$OS" in
        darwin)
            PLATFORM="apple-darwin"
            ;;
        linux)
            PLATFORM="unknown-linux-gnu"
            ;;
        mingw*|msys*|cygwin*)
            PLATFORM="pc-windows-msvc"
            ;;
        *)
            error "Unsupported OS: $OS"
            ;;
    esac

    TARGET="${ARCH}-${PLATFORM}"
}

# Check for required commands
check_requirements() {
    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        error "curl or wget is required but not installed"
    fi
    if ! command -v tar >/dev/null 2>&1; then
        error "tar is required but not installed"
    fi
}

# Download a file
download() {
    url="$1"
    output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    fi
}

# Get the download URL for a release
get_download_url() {
    if [ "$FORGE_VERSION" = "latest" ]; then
        echo "https://github.com/${GITHUB_REPO}/releases/latest/download/forge-${TARGET}.tar.gz"
    else
        echo "https://github.com/${GITHUB_REPO}/releases/download/${FORGE_VERSION}/forge-${TARGET}.tar.gz"
    fi
}

# Add to PATH in shell rc files
add_to_path() {
    shell_rc="$1"
    path_line='export PATH="$HOME/.forge/bin:$PATH"'

    if [ -f "$shell_rc" ]; then
        if ! grep -q ".forge/bin" "$shell_rc" 2>/dev/null; then
            printf "\n# Forge\n%s\n" "$path_line" >> "$shell_rc"
            info "Added Forge to $shell_rc"
        fi
    fi
}

# Main installation
main() {
    echo ""
    echo "  Forge Installer"
    echo "  ==============="
    echo ""

    check_requirements
    detect_platform

    info "Detected platform: ${TARGET}"
    info "Installing to: ${INSTALL_DIR}"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    # Download
    DOWNLOAD_URL=$(get_download_url)
    info "Downloading from: ${DOWNLOAD_URL}"

    ARCHIVE="$TMP_DIR/forge.tar.gz"
    if ! download "$DOWNLOAD_URL" "$ARCHIVE"; then
        error "Download failed. Please check your internet connection and try again."
    fi

    # Extract
    info "Extracting..."
    tar -xzf "$ARCHIVE" -C "$INSTALL_DIR"

    # Make binaries executable
    chmod +x "$INSTALL_DIR/forge" 2>/dev/null || true
    chmod +x "$INSTALL_DIR/forge-host" 2>/dev/null || true

    # Verify installation
    if [ ! -f "$INSTALL_DIR/forge" ]; then
        error "Installation failed: forge binary not found"
    fi
    if [ ! -f "$INSTALL_DIR/forge-host" ]; then
        error "Installation failed: forge-host binary not found"
    fi

    # Add to PATH
    add_to_path "$HOME/.bashrc"
    add_to_path "$HOME/.zshrc"
    add_to_path "$HOME/.profile"

    # Success message
    echo ""
    success "Forge installed successfully!"
    echo ""
    echo "  Installed:"
    echo "    - $INSTALL_DIR/forge"
    echo "    - $INSTALL_DIR/forge-host"
    echo ""
    echo "  To get started, restart your terminal or run:"
    echo ""
    echo "    export PATH=\"\$HOME/.forge/bin:\$PATH\""
    echo ""
    echo "  Then create your first app:"
    echo ""
    echo "    forge init my-app"
    echo "    cd my-app"
    echo "    forge dev ."
    echo ""
}

main "$@"
