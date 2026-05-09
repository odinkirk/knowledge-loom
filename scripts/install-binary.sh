#!/usr/bin/env bash
set -euo pipefail

# Knowledge Loom Binary Installer
# Detects OS and architecture, downloads the latest release binary

REPO="odinkirk/knowledge-loom"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.knowledge-loom/bin}"
VERSION="${VERSION:-latest}"

echo "==> Installing Knowledge Loom..."

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-unknown-linux-gnu"
                ;;
            aarch64|arm64)
                TARGET="aarch64-unknown-linux-gnu"
                ;;
            *)
                echo "Error: Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-apple-darwin"
                ;;
            arm64|aarch64)
                TARGET="aarch64-apple-darwin"
                ;;
            *)
                echo "Error: Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-pc-windows-msvc"
                ;;
            *)
                echo "Error: Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Error: Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Detected platform: $OS $ARCH"
echo "Target: $TARGET"

# Get latest release info
if [ "$VERSION" = "latest" ]; then
    echo "Fetching latest release info..."
    RELEASE_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url" | grep "$TARGET" | head -1 | cut -d '"' -f 4)
else
    RELEASE_URL="https://github.com/$REPO/releases/download/$VERSION/loom-$OS-$ARCH.tar.gz"
fi

if [ -z "$RELEASE_URL" ]; then
    echo "Error: Could not find release for platform $OS $ARCH"
    exit 1
fi

echo "Downloading from: $RELEASE_URL"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download and extract
if [ "$OS" = "Linux" ] || [ "$OS" = "Darwin" ]; then
    TEMP_FILE=$(mktemp)
    curl -L "$RELEASE_URL" -o "$TEMP_FILE"
    tar xzf "$TEMP_FILE" -C "$INSTALL_DIR"
    rm "$TEMP_FILE"
    echo "Extracted binary to: $INSTALL_DIR/loom"
elif [ "$OS" = "MINGW" ] || [ "$OS" = "MSYS" ] || [ "$OS" = "CYGWIN" ]; then
    TEMP_FILE=$(mktemp).zip
    curl -L "$RELEASE_URL" -o "$TEMP_FILE"
    unzip -o "$TEMP_FILE" -d "$INSTALL_DIR"
    rm "$TEMP_FILE"
    echo "Extracted binary to: $INSTALL_DIR/loom.exe"
fi

# Make executable
if [ "$OS" != "MINGW" ] && [ "$OS" != "MSYS" ] && [ "$OS" != "CYGWIN" ]; then
    chmod +x "$INSTALL_DIR/loom"
fi

echo ""
echo "==> Installation complete!"
echo "Binary installed to: $INSTALL_DIR/loom"
echo ""
echo "Next steps:"
echo "  1. Add $INSTALL_DIR to your PATH"
echo "  2. Run: loom init"
echo "  3. Restart your coding agent"
echo ""
echo "For more information, visit: https://github.com/$REPO"