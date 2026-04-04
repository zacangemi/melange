#!/bin/sh
set -e

REPO="zacangemi/melange"
INSTALL_DIR="$HOME/.melange/bin"
BINARY="melange"

cat << 'ART'

               ~~~~                @@@@@@@@@@@@@@                ~~~~
                              @@@@@@@@@@@@@@@@@@@@@@@@
                           @@@@@@@ \ \ \ \ | / / / / @@@@@@@
                         @@@@@@ \ \ \ \ \ \ | / / / / / / @@@@@@
                       @@@@@ \ \ \ \ \ \ \ \ | / / / / / / / / @@@@@
                      @@@@ \ \ \ \ \ \ \ \ \ | / / / / / / / / / @@@@
                     @@@@ \ \ \ \ \ \ \ \ \\ | // / / / / / / / / @@@@
                    @@@@ - \ \ \ \ \ \ \\\\ | //// / / / / / / - @@@@
                   @@@@ - - \ \ \ \ \\\\\\  |  ////// / / / / - - @@@@
                   @@@@ - - - \ \ \\\\\\\\     //////// / / - - - @@@@
                  @@@@ - - - - \ \\\\\\\\\\  .  ////////// / - - - - @@@@
                  @@@@ - - - - - \\\\\\\\\\  .  ////////// - - - - - @@@@
                  @@@@ - - - - - - - \\\\\  . .  ///// - - - - - - - @@@@
                  @@@@ - - - - - - - - - -  . .  - - - - - - - - - - @@@@
                  @@@@ - - - - - - - /////  . .  \\\\\ - - - - - - - @@@@
                  @@@@ - - - - - //////////  .  \\\\\\\\\\ - - - - - @@@@
                   @@@@ - - - / / ////////     \\\\\\\\ \ \ - - - @@@@
  ~~                @@@@ - - / / / / //////  |  \\\\\\ \ \ \ \ - - @@@@                ~~
  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~
  ~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~
      .  i i .    i i .    i i .    i i      i i .    i i .    i i .    i i .

                                   o         o
                                  /|\       /|\
                                  / \       / \

ART
echo 'M E L A N G E  Installer'
echo '"The memory must flow..."'
echo ''

# --- Platform checks ---

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

# --- Warn about old install ---

if [ -f "/usr/local/bin/melange" ]; then
    echo "Note: Found old install at /usr/local/bin/melange"
    echo "      You can remove it with: sudo rm /usr/local/bin/melange"
    echo ""
fi

# --- Create install directory ---

mkdir -p "$INSTALL_DIR"

# --- Try pre-built binary, fall back to source ---

echo "Fetching latest release..."
DOWNLOAD_URL=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"browser_download_url"' | grep 'aarch64-apple-darwin' | head -1 | cut -d '"' -f 4 || true)

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

    echo "Building (this takes about 30 seconds)..."
    cargo build --release --jobs 4

    echo "Installing to ${INSTALL_DIR}..."
    cp "target/release/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"

    # Cleanup
    rm -rf "$TMPDIR"
else
    # Download pre-built binary
    echo "Downloading pre-built binary..."
    TMPDIR=$(mktemp -d)
    curl -sSL "$DOWNLOAD_URL" -o "$TMPDIR/melange.tar.gz"
    tar -xzf "$TMPDIR/melange.tar.gz" -C "$TMPDIR"

    echo "Installing to ${INSTALL_DIR}..."
    mv "$TMPDIR/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
    rm -rf "$TMPDIR"
fi

# --- Update PATH in shell rc file ---

PATH_LINE="export PATH=\"\$HOME/.melange/bin:\$PATH\""

add_to_path() {
    RC_FILE="$1"
    if [ -f "$RC_FILE" ]; then
        if ! grep -qF '.melange/bin' "$RC_FILE"; then
            echo "" >> "$RC_FILE"
            echo "# Melange" >> "$RC_FILE"
            echo "$PATH_LINE" >> "$RC_FILE"
            echo "  Added to PATH in $RC_FILE"
            return 0
        else
            echo "  PATH already configured in $RC_FILE"
            return 0
        fi
    fi
    return 1
}

echo ""
FOUND_RC=0

# Try shell rc files in order of preference
if [ -n "$SHELL" ]; then
    case "$SHELL" in
        */zsh)
            add_to_path "$HOME/.zshrc" && FOUND_RC=1
            ;;
        */bash)
            add_to_path "$HOME/.bashrc" && FOUND_RC=1
            if [ "$FOUND_RC" -eq 0 ]; then
                add_to_path "$HOME/.bash_profile" && FOUND_RC=1
            fi
            ;;
    esac
fi

# Fallback: try common rc files
if [ "$FOUND_RC" -eq 0 ]; then
    add_to_path "$HOME/.zshrc" && FOUND_RC=1
fi
if [ "$FOUND_RC" -eq 0 ]; then
    add_to_path "$HOME/.bashrc" && FOUND_RC=1
fi
if [ "$FOUND_RC" -eq 0 ]; then
    add_to_path "$HOME/.profile" && FOUND_RC=1
fi
if [ "$FOUND_RC" -eq 0 ]; then
    echo "  Could not find a shell rc file to update."
    echo "  Add this to your shell profile manually:"
    echo "    $PATH_LINE"
fi

# Make melange available immediately in this shell
export PATH="$HOME/.melange/bin:$PATH"

# --- Install shell completions ---

mkdir -p "$HOME/.melange/completions"
"${INSTALL_DIR}/${BINARY}" completions zsh > "$HOME/.melange/completions/_melange" 2>/dev/null || true

# Add completions to fpath in zshrc
FPATH_LINE='fpath=(~/.melange/completions $fpath)'
if [ -f "$HOME/.zshrc" ]; then
    if ! grep -qF '.melange/completions' "$HOME/.zshrc"; then
        echo "$FPATH_LINE" >> "$HOME/.zshrc"
        echo "autoload -Uz compinit && compinit" >> "$HOME/.zshrc"
        echo "  Shell completions installed (restart your shell to activate)"
    fi
fi

echo ""
echo "============================================"
echo "  Installed successfully!"
echo "============================================"
echo ""
echo "  Run it now:"
echo ""
echo "    melange              # Launch the TUI dashboard"
echo "    melange config       # Configure model directory"
echo ""
echo "The spice must flow."
