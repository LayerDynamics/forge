#!/usr/bin/env bash
#
# Run the documentation drift gate locally. Exits non-zero (with a punch-list)
# if the docs site has drifted from the code. Mirrors the `Docs Sync` CI job and
# the `docs_in_sync` test.
#
# Usage:
#   scripts/docs-check.sh            # report drift, exit non-zero on drift
#   UPDATE_DOCS_BASELINE=1 scripts/docs-check.sh   # re-record the known-drift baseline
set -euo pipefail

# Resolve the workspace root from this script's location so it works from any CWD.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

if [[ "${UPDATE_DOCS_BASELINE:-}" == "1" ]]; then
  echo "Re-recording the known-drift baseline..."
  UPDATE_DOCS_BASELINE=1 cargo test -q -p forge-docs-check --test docs_sync
  exit 0
fi

exec cargo run -q -p forge-docs-check
