# Forge developer convenience targets.
# CI enforces fmt, clippy (-D warnings), tests, and the documentation drift gate.

.PHONY: docs-check docs-report docs-api docs-counts docs-crates docs-examples docs-baseline install-hooks fmt clippy test check ci

# Documentation drift gate (baseline-aware): fails only on NEW drift beyond the
# recorded baseline. Same logic as CI and the pre-commit hook.
docs-check:
	cargo test -q -p forge-docs-check --test docs_sync

# Regenerate every <!-- forge:api --> signature block in place from the SDK
# (marker-hybrid generator). Bespoke prose outside the markers is untouched.
docs-api:
	cargo run -q -p forge-docs-check -- --write-api-blocks

# Regenerate every <!-- forge:count:* --> marker in the docs to the current
# derived workspace counts (ext crates / total crates / runtime modules).
docs-counts:
	cargo run -q -p forge-docs-check -- --write-counts

# Generate a Starlight page for any crate that lacks one (from its //! module
# doc + Cargo.toml). Gap-fill only: never overwrites an existing crate page.
docs-crates:
	cargo run -q -p forge-docs-check -- --write-crate-pages

# Regenerate every <!-- forge:example --> block from each example app's
# runtime:* imports.
docs-examples:
	cargo run -q -p forge-docs-check -- --write-example-blocks

# Full drift report: list ALL current drift (fails if any exists at all).
# Useful for seeing the whole backlog; not the burn-down gate.
docs-report:
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
