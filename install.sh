#!/bin/bash
set -e

# Install GPT Engage to ~/.local/bin
# This follows standard Unix conventions and ~/.local/bin is often in PATH

INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="gptengage"

echo "Building GPT Engage in release mode..."
cargo build --release

echo "Creating install directory: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

echo "Installing gptengage to $INSTALL_DIR/$BINARY_NAME"
cp target/release/gptengage "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo ""
echo "✓ GPT Engage installed successfully!"
echo ""
echo "Installation path: $INSTALL_DIR/$BINARY_NAME"
echo ""

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
    echo "✓ $HOME/.local/bin is already in your PATH"
    echo ""
    echo "You can now run: gptengage --help"
else
    echo "⚠ $HOME/.local/bin is NOT in your PATH"
    echo ""
    echo "Add this line to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then run: source ~/.bashrc (or source ~/.zshrc)"
    echo ""
    echo "Or run gptengage with full path: $INSTALL_DIR/$BINARY_NAME"
fi

echo ""
echo "Detecting available CLIs..."
echo ""

# Check for available CLIs
if command -v claude &> /dev/null; then
    echo "✓ Claude Code CLI detected"
else
    echo "✗ Claude Code CLI not found (optional)"
fi

if command -v codex &> /dev/null; then
    echo "✓ Codex CLI detected"
else
    echo "✗ Codex CLI not found (optional)"
fi

if command -v gemini &> /dev/null; then
    echo "✓ Gemini CLI detected"
else
    echo "✗ Gemini CLI not found (optional)"
fi

echo ""
echo "To get started:"
echo "  gptengage status              # Show available CLIs and sessions"
echo "  gptengage --help              # Show all commands"
echo "  gptengage invoke claude \"hello\"  # Invoke a CLI"
echo "  gptengage debate \"topic here\"    # Run a multi-AI debate"
echo ""
echo "For more information:"
echo "  https://github.com/yourusername/gptengage"
echo ""
