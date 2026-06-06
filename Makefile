# Forge developer convenience targets.
# CI enforces fmt, clippy (-D warnings), tests, and the documentation drift gate.

.PHONY: docs-check docs-baseline install-hooks fmt clippy test check ci

# Fail if the documentation site has drifted from the code (prints a punch-list).
docs-check:
	cargo run -q -p forge-docs-check

# Re-record the known-drift baseline after intentionally changing docs/code.
docs-baseline:
	UPDATE_DOCS_BASELINE=1 cargo test -q -p forge-docs-check --test docs_sync

# Install the git pre-commit hook that runs the drift gate locally.
install-hooks:
	scripts/install-hooks.sh

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace -- -D warnings

test:
	cargo test --workspace

check:
	cargo check --workspace

# Everything CI runs, locally.
ci: fmt clippy test docs-check
	cargo fmt --all -- --check
