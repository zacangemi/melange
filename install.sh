#!/bin/sh
set -e

REPO="zacangemi/melange"
INSTALL_DIR="/usr/local/bin"
BINARY="melange"

echo ''
echo '    ___'
echo '___/ o \____'
echo '   ___     \___________'
echo '  /   \___             \'
echo '           \___   ___  |'
echo '               \_/  \_|'
echo ''
echo 'M E L A N G E  Installer'
echo '"The memory must flow..."'
echo ''

# Check architecture
ARCH=$(uname -m)
OS=$(uname -s)

if [ "$OS" != "Darwin" ]; then
    echo "Error: Melange currently only supports macOS."
    echo "Linux support coming in a future phase."
    exit 1
fi

if [ "$ARCH" != "arm64" ]; then
    echo "Error: Melange requires Apple Silicon (M1/M2/M3/M4)."
    echo "Detected architecture: $ARCH"
    exit 1
fi

# Get latest release
echo "Fetching latest release..."
DOWNLOAD_URL=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"browser_download_url"' | grep 'melange-macos-arm64' | head -1 | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "No pre-built binary found. Building from source..."
    echo ""

    # Check for Rust
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        . "$HOME/.cargo/env"
    fi

    # Clone and build
    TMPDIR=$(mktemp -d)
    echo "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "$TMPDIR/melange"
    cd "$TMPDIR/melange"

    echo "Building (this takes about 10 seconds)..."
    cargo build --release

    echo "Installing to ${INSTALL_DIR}..."
    sudo cp "target/release/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY}"

    # Cleanup
    rm -rf "$TMPDIR"
else
    # Download pre-built binary
    echo "Downloading pre-built binary..."
    TMPFILE=$(mktemp)
    curl -sSL "$DOWNLOAD_URL" -o "$TMPFILE"
    chmod +x "$TMPFILE"

    echo "Installing to ${INSTALL_DIR}..."
    sudo mv "$TMPFILE" "${INSTALL_DIR}/${BINARY}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY}"
fi

echo ""
echo "Installed successfully!"
echo ""
echo "Run it:"
echo "  melange              # Launch the TUI dashboard"
echo "  melange --json       # Output as JSON"
echo "  melange --scan PATH  # Custom model directory"
echo ""
echo "The spice must flow."
