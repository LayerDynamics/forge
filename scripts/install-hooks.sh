#!/usr/bin/env bash
#
# Install a git pre-commit hook that runs the documentation drift gate, so drift
# is caught locally before it ever reaches CI. Bypassable with `git commit
# --no-verify` for work-in-progress commits; CI remains the hard backstop.
#
# Usage: scripts/install-hooks.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOK="$ROOT/.git/hooks/pre-commit"

if [[ ! -d "$ROOT/.git" ]]; then
  echo "error: $ROOT is not a git repository (no .git directory)" >&2
  exit 1
fi

cat > "$HOOK" <<'HOOK_EOF'
#!/usr/bin/env bash
# Forge pre-commit hook: documentation drift gate.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
echo "[pre-commit] checking documentation sync..."
if ! "$ROOT/scripts/docs-check.sh"; then
  echo "" >&2
  echo "[pre-commit] documentation drift detected (see above)." >&2
  echo "  Fix the docs / run the generators, or commit with --no-verify to bypass." >&2
  exit 1
fi
HOOK_EOF

chmod +x "$HOOK"
echo "Installed pre-commit hook -> $HOOK"
