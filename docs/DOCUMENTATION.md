# Documentation Generation Guide

Forge uses two documentation systems for different purposes. This guide explains how to generate documentation that excludes external dependencies and focuses only on your Forge code.

## 1. Rust API Documentation (`cargo doc`)

Generates documentation for Rust crates using rustdoc. This is useful for:
- Internal Rust API reference
- Understanding Forge's Rust implementation
- Contributing to Forge development

### Generate Docs

```bash
# Generate docs for workspace crates only (no dependencies)
cargo doc --workspace --no-deps

# Generate and open in browser
cargo doc --workspace --no-deps --open

# Use convenience script
./scripts/generate-rust-docs.sh
```

### Output Location

- **Directory:** `target/doc/`
- **Entry points:**
  - `target/doc/forge_cli/index.html` - CLI tool
  - `target/doc/forge_runtime/index.html` - Runtime
  - `target/doc/ext_fs/index.html` - File system extension
  - `target/doc/ext_window/index.html` - Window extension
  - etc.

### Configuration

Documentation behavior is configured in:
- `.cargo/config.toml` - Build flags and rustdoc settings
- `Cargo.toml` - Workspace metadata for docs.rs
- Individual `crates/*/Cargo.toml` - Package-specific metadata

**Important:** Always use the `--no-deps` flag to exclude dependency documentation. This flag:
- Reduces build time significantly
- Keeps documentation focused on Forge code
- Prevents clutter from third-party crates
- Cannot be configured in config files (must use CLI or scripts)

### Customization

To customize rustdoc output, you can:
1. Edit `.cargo/config.toml` to add global rustdoc flags
2. Add package-specific flags in `Cargo.toml` under `[package.metadata.docs.rs]`
3. Pass additional flags: `cargo doc --workspace --no-deps -- --additional-flag`

## 2. TypeScript/Rust API Documentation (`forge docs`)

Generates documentation for Forge's public TypeScript APIs using the custom `forge-etch` system. This is useful for:
- App developer reference
- Extension API documentation
- Website documentation generation

### Generate Docs

```bash
# Document all extensions
forge docs --all-extensions

# Document specific extension
forge docs --extension fs -o docs/api/fs

# Document an app
forge docs my-app

# Choose output format
forge docs --all-extensions --format html        # HTML only
forge docs --all-extensions --format astro       # Astro markdown (default)
forge docs --all-extensions --format both        # Both formats
```

### Output Location

Configurable with `--output` flag (default: `docs/`)

### How It Works

The `forge docs` command:
1. Parses TypeScript from `ts/init.ts` files
2. Parses Rust from `src/lib.rs` files
3. Extracts public API signatures and documentation comments
4. Generates Astro markdown or HTML documentation
5. **Automatically excludes all dependencies** - only processes Forge source files

Unlike `cargo doc`, the `forge docs` command never processes dependency code because it only looks at specific source files in your Forge workspace.

## Best Practices

### 1. Always Exclude Dependencies

**For Rust documentation:**
```bash
# ✅ Correct - excludes dependencies
cargo doc --workspace --no-deps

# ❌ Incorrect - includes all dependencies
cargo doc --workspace
```

**For Forge documentation:**
Dependencies are automatically excluded - no configuration needed!

### 2. Keep Documentation in Sync

- Update documentation comments when changing code
- Run `cargo doc` locally before pushing to catch doc errors
- Consider adding a docs generation check to CI

### 3. Use Appropriate Tools

- Use `cargo doc` for internal Rust API reference
- Use `forge docs` for public TypeScript API documentation
- Use both when documenting extensions that have both Rust and TypeScript interfaces

### 4. Optimize Build Time

Documentation generation can be slow. To speed it up:
- Always use `--no-deps` with `cargo doc`
- Use `--document-private-items` only when needed
- Consider caching `target/doc/` in CI
- For quick checks, document only specific packages: `cargo doc -p ext_fs --no-deps`

### 5. CI Integration

Add documentation generation to your CI pipeline:

```yaml
- name: Check documentation builds
  run: cargo doc --workspace --no-deps --document-private-items
```

This ensures:
- Documentation comments are valid
- No broken links in docs
- Consistency across all crates

## Common Issues

### Issue: Documentation includes dependencies

**Solution:** Always use the `--no-deps` flag with `cargo doc`:
```bash
cargo doc --workspace --no-deps
```

### Issue: Documentation build is slow

**Solutions:**
1. Use `--no-deps` to exclude dependencies
2. Document specific packages: `cargo doc -p forge_cli --no-deps`
3. Cache `target/doc/` in CI environments

### Issue: Missing documentation for private items

**Solution:** Use `--document-private-items` flag:
```bash
cargo doc --workspace --no-deps --document-private-items
```

This is already configured in `.cargo/config.toml` but may need to be passed explicitly in some contexts.

### Issue: forge docs not found

**Solution:** Make sure you've built the CLI:
```bash
cargo build -p forge_cli
```

Or use via cargo:
```bash
cargo run -p forge_cli -- docs --all-extensions
```

## Configuration Reference

### `.cargo/config.toml`

```toml
[build]
rustdocflags = ["--document-private-items"]

[doc]
browser = ["open"]  # Browser for --open flag
```

### Workspace `Cargo.toml`

```toml
[workspace.metadata.docs.rs]
all-features = true
rustdoc-args = ["--document-private-items"]
```

### Package `Cargo.toml`

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--document-private-items"]
```

## Summary

**Rust Documentation (`cargo doc`):**
- Must use `--no-deps` flag to exclude dependencies
- Configured via `.cargo/config.toml` and `Cargo.toml` files
- Outputs to `target/doc/`
- Use for internal Rust API reference

**Forge Documentation (`forge docs`):**
- Automatically excludes dependencies (no configuration needed)
- Parses TypeScript and Rust source directly
- Outputs to `docs/` (configurable)
- Use for public API documentation

**Key Takeaway:** Both systems can generate dependency-free documentation, but they use different approaches. `cargo doc` requires the `--no-deps` flag, while `forge docs` excludes dependencies by design.
