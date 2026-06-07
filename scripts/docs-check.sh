#!/usr/bin/env bash
#
# Documentation drift gate (local). Shows the full current drift report, then
# enforces the baseline: exits non-zero only on NEW drift (drift beyond what is
# recorded in crates/forge-docs-check/tests/known_drift_baseline.txt). This
# mirrors the `Docs Sync` CI job and the `docs_in_sync` test, so it does not
# block commits on the known backlog that later phases will burn down.
#
# Usage:
#   scripts/docs-check.sh                          # report + enforce baseline
#   UPDATE_DOCS_BASELINE=1 scripts/docs-check.sh   # re-record the baseline
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

# Full report (informational — the binary lists ALL drift and never gates here).
cargo run -q -p forge-docs-check || true

echo
echo "Enforcing baseline (fails only on NEW drift)..."
exec cargo test -q -p forge-docs-check --test docs_sync
