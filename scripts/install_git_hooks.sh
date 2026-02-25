#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo ".")
GIT_DIR="$REPO_ROOT/.git"
COMMITHOOKS="${COMMITHOOKS_DIR:-$HOME/Documents/commithooks}"

echo "Installing commithooks dispatchers..."

if [ ! -d "$COMMITHOOKS/lib" ]; then
    echo "Commithooks source not found at $COMMITHOOKS" >&2
    echo "Set COMMITHOOKS_DIR or clone to ~/Documents/commithooks/" >&2
    exit 1
fi

mkdir -p "$GIT_DIR/hooks"

for hook in pre-commit commit-msg pre-push post-checkout post-merge; do
    src="$COMMITHOOKS/$hook"
    if [ -f "$src" ]; then
        cp "$src" "$GIT_DIR/hooks/$hook"
        chmod +x "$GIT_DIR/hooks/$hook"
        echo "  [ok] $hook"
    fi
done

rm -rf "${GIT_DIR:?}/lib"
cp -r "$COMMITHOOKS/lib" "$GIT_DIR/lib"
echo "  [ok] lib/ ($(ls "$GIT_DIR/lib/" | wc -l) modules)"

# Unset core.hooksPath if set (we use .git/hooks/ directly)
if git config --get core.hooksPath >/dev/null 2>&1; then
    git config --unset core.hooksPath
    echo "  [ok] Unset core.hooksPath"
fi

echo ""
echo "Commithooks installed from $COMMITHOOKS"
echo ""
echo "Active hooks:"
echo "  pre-commit   -> scripts/git-hooks/pre-commit (fmt, clippy, secrets)"
echo "  commit-msg   -> scripts/git-hooks/commit-msg (message format)"
echo "  pre-push     -> scripts/git-hooks/pre-push (trufflehog, audit, deny)"
echo "  post-checkout, post-merge -> no-op (add .githooks/ stubs to enable)"
echo ""
echo "To bypass hooks (not recommended):"
echo "  git commit --no-verify"
