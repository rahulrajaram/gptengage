#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo ".")
HOOKS_DIR="$REPO_ROOT/.git/hooks"
SCRIPTS_DIR="$REPO_ROOT/scripts/git-hooks"

echo "Installing Git hooks..."

# Ensure .git/hooks directory exists
mkdir -p "$HOOKS_DIR"

# Install pre-commit hook
if [ -f "$SCRIPTS_DIR/pre-commit" ]; then
    ln -sf "../../scripts/git-hooks/pre-commit" "$HOOKS_DIR/pre-commit"
    echo "✅ Installed pre-commit hook"
else
    echo "⚠️  pre-commit hook not found at $SCRIPTS_DIR/pre-commit" >&2
fi

# Install commit-msg hook
if [ -f "$SCRIPTS_DIR/commit-msg" ]; then
    ln -sf "../../scripts/git-hooks/commit-msg" "$HOOKS_DIR/commit-msg"
    echo "✅ Installed commit-msg hook"
else
    echo "⚠️  commit-msg hook not found at $SCRIPTS_DIR/commit-msg" >&2
fi

echo ""
echo "Git hooks installed successfully!"
echo ""
echo "The following hooks are now active:"
echo "  • pre-commit  - Runs cargo fmt, clippy, and secret scanning"
echo "  • commit-msg  - Validates commit message format"
echo ""
echo "To bypass hooks (not recommended):"
echo "  git commit --no-verify"
