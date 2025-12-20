# Forge Documentation Style Guide

Standards and conventions for documenting Forge codebase to ensure consistency, clarity, and maintainability.

## Table of Contents

1. [General Principles](#general-principles)
2. [Tone and Voice](#tone-and-voice)
3. [Documentation Structure](#documentation-structure)
4. [Code Examples](#code-examples)
5. [ASCII Diagrams](#ascii-diagrams)
6. [Async Functions](#async-functions)
7. [Cross-Referencing](#cross-referencing)
8. [Naming Conventions](#naming-conventions)
9. [Common Pitfalls](#common-pitfalls)
10. [Quality Checklist](#quality-checklist)

---

## General Principles

### 1. Write for the Reader

Documentation should be written from the **user's perspective**, not the implementer's.

**Good:**
```rust
/// Reads the entire contents of a file as a UTF-8 string
///
/// # Errors
///
/// Returns an error if the file doesn't exist or contains invalid UTF-8.
```

**Bad:**
```rust
/// This function opens a file descriptor, allocates a buffer, and reads...
```

### 2. Be Concise but Complete

- **First line**: One-line summary (appears in autocomplete)
- **Body**: 2-3 sentences of context
- **Sections**: Use standard sections (`# Errors`, `# Examples`, etc.)

### 3. Show, Don't Just Tell

Always prefer concrete examples over abstract descriptions.

**Good:**
```rust
/// # Examples
///
/// ```no_run
/// let config = Config::load("app.toml")?;
/// println!("Version: {}", config.version);
/// ```
```

**Bad:**
```rust
/// You can use this to load configuration files.
```

### 4. Document the "Why" Not Just the "What"

Explain **why** something exists or works a certain way, especially for non-obvious design decisions.

**Good:**
```rust
/// Extension initialization happens in tiers to ensure simpler extensions
/// (like timers) are ready before complex ones (like IPC) that depend on them.
```

**Bad:**
```rust
/// Extensions are initialized by tier.
```

---

## Tone and Voice

### Active Voice

Use active voice for clarity and directness.

**Good:**
- "Returns an error if..."
- "Creates a new window with..."
- "Validates the manifest and builds..."

**Bad:**
- "An error is returned if..."
- "A new window is created with..."
- "The manifest is validated and built..."

### Present Tense

Describe functionality in present tense.

**Good:**
- "This function reads the file"
- "Returns the parsed configuration"

**Bad:**
- "This function will read the file"
- "Will return the parsed configuration"

### Direct Address

Address the reader as "you" when appropriate.

**Good:**
```rust
/// If you need to customize the behavior, use the builder pattern:
```

**Acceptable:**
```rust
/// Use the builder pattern to customize the behavior:
```

**Avoid:**
```rust
/// Users should use the builder pattern...
```

---

## Documentation Structure

### Module-Level Documentation (`//!`)

**Order:**
1. One-line description
2. Overview paragraph (2-3 sentences)
3. Architecture section (with diagram if helpful)
4. Key concepts
5. Usage example
6. Implementation details (if relevant)
7. See also / Examples directory

**Example:**
```rust
//! Module Name - Brief description
//!
//! Detailed overview of what this module provides.
//!
//! # Architecture
//!
//! [ASCII diagram if helpful]
//!
//! # Usage
//!
//! ```no_run
//! use crate::module;
//! ```
```

### Item Documentation (`///`)

**Order for functions:**
1. One-line summary
2. Context paragraph (2-3 sentences)
3. `# Parameters` (if not obvious)
4. `# Returns`
5. `# Errors`
6. `# Panics` (if applicable)
7. `# Examples`
8. `# See Also` (optional)

### Standard Section Headers

Use these exact header names (case-sensitive):

- `# Parameters` - Function parameters
- `# Returns` - Return value description
- `# Errors` - Error cases
- `# Panics` - Panic conditions
- `# Safety` - Safety requirements (for unsafe code)
- `# Examples` - Usage examples
- `# See Also` - Related functionality

---

## Code Examples

### Use `no_run` for Most Examples

Unless the example is self-contained and runs without external dependencies, use `no_run`:

```rust
/// # Examples
///
/// ```no_run
/// let window = Window::new("My App")?;
/// window.show();
/// ```
```

### When to Use Other Attributes

| Attribute | Use Case | Example |
|-----------|----------|---------|
| `no_run` | Requires external setup, files, or state | File I/O, window creation |
| `ignore` | Example is illustrative only, won't compile | Pseudo-code, partial snippets |
| `should_panic` | Demonstrates panic behavior | Validation failures |
| *none* | Self-contained, runs in doctest | Pure functions, math |

### Show Error Handling

**Good:**
```rust
/// ```no_run
/// let config = Config::load("app.toml")?;
/// // Use config...
/// ```
```

**Also Good:**
```rust
/// ```no_run
/// let config = Config::load("app.toml").unwrap();
/// ```
```

**Bad:**
```rust
/// ```ignore
/// let config = Config::load("app.toml");
/// ```
```

### Complete Examples

Examples should be complete enough to understand context:

**Good:**
```rust
/// ```no_run
/// use forge_weld::ExtensionBuilder;
///
/// ExtensionBuilder::new("runtime_fs", "runtime:fs")
///     .ts_path("ts/init.ts")
///     .ops(&["op_fs_read"])
///     .build()?;
/// ```
```

**Bad:**
```rust
/// ```ignore
/// builder.ops(&["op_fs_read"])
/// ```
```

---

## ASCII Diagrams

### When to Use ASCII Diagrams

Use ASCII diagrams for:

1. **Architecture overviews** - Component relationships
2. **Data flow** - IPC, event loops, request/response cycles
3. **State machines** - Extension tiers, initialization sequences
4. **Directory structures** - Build outputs, project layouts

**Don't use diagrams for:**
- Simple relationships (use lists instead)
- Single-direction flows (use numbered lists)
- Complex graphs (link to external diagrams)

### Diagram Style Guidelines

**1. Use box-drawing characters:**

```text
┌─────────────┐
│   Component │
└──────┬──────┘
       │
   ┌───▼───┐
   │ Other │
   └───────┘
```

**2. Keep diagrams simple:**
- Maximum 5-7 components
- One concept per diagram
- Clear direction indicators (arrows, ▼, ►)

**3. Label everything:**

**Good:**
```text
┌─────────────────────────────────────────────────┐
│              Tao Event Loop (OS)                │
└───────────┬─────────────────────┬───────────────┘
            │                     │
    ┌───────▼────────┐    ┌──────▼─────────┐
    │ Deno JsRuntime │    │  Wry WebView   │
    │  (src/main.ts) │◄──►│ (web/index.html)│
    └────────────────┘    └────────────────┘
```

**Bad:**
```text
A -> B -> C
```

**4. Align consistently:**
```text
    Module A          Module B
       │                 │
       ▼                 ▼
   Component         Component
```

---

## Async Functions

### Document Async Behavior

Always mention that a function is async and what that means:

**Good:**
```rust
/// Reads the entire file asynchronously
///
/// This function returns immediately and performs I/O on the background
/// thread pool. The caller must `.await` the returned future.
///
/// # Examples
///
/// ```no_run
/// let contents = read_file("data.txt").await?;
/// ```
#[weld_op(async)]
#[op2(async)]
pub async fn op_read_file(path: String) -> Result<String, Error> {
    // ...
}
```

### TypeScript Documentation

When documenting ops that become TypeScript functions, show the TypeScript usage:

```rust
/// Reads a file's contents
///
/// # TypeScript Usage
///
/// ```typescript
/// import { readFile } from "runtime:fs";
///
/// const text = await readFile("./data.txt");
/// ```
```

---

## Cross-Referencing

### Use Backtick Links Liberally

Link to types, functions, and modules using backtick syntax:

```rust
/// See [`ExtensionBuilder`] for configuration options.
/// Use [`Self::new()`] to create an instance.
/// Errors are defined in [`crate::errors`].
```

### Linking Syntax

| Target | Syntax | Example |
|--------|--------|---------|
| Same module | `` [`Type`] `` | `` [`Config`] `` |
| Other module | `` [`module::Type`] `` | `` [`fs::FileInfo`] `` |
| Crate root | `` [`crate::Type`] `` | `` [`crate::Error`] `` |
| Current type method | `` [`Self::method()`] `` | `` [`Self::build()`] `` |
| Trait method | `` [`Trait::method()`] `` | `` [`Display::fmt()`] `` |

### Linking Error Variants

```rust
/// # Errors
///
/// - [`Error::NotFound`] - File doesn't exist
/// - [`Error::PermissionDenied`] - No read permission
```

---

## Naming Conventions

### Rust ↔ TypeScript Field Names

Rust uses `snake_case`, TypeScript uses `camelCase`:

| Rust | TypeScript |
|------|------------|
| `user_id` | `userId` |
| `top_left` | `topLeft` |
| `is_enabled` | `isEnabled` |

**Document this conversion when relevant:**

```rust
/// Window position
///
/// Note: In TypeScript, this appears as `topLeft` (camelCase).
pub top_left: Point,
```

### Function Name Conventions

| Pattern | Example | Purpose |
|---------|---------|---------|
| `op_*` | `op_fs_read` | Deno core operation |
| `cmd_*` | `cmd_build` | CLI command handler |
| `check_*` | `check_permission` | Validation function |
| `init_*` | `init_extensions` | Initialization function |
| `find_*` | `find_binary` | Search function |
| `create_*` | `create_window` | Factory function |

### Constant Naming

```rust
/// Maximum number of concurrent connections
///
/// This limit prevents resource exhaustion.
const MAX_CONNECTIONS: usize = 100;
```

---

## Common Pitfalls

### ❌ Don't Repeat the Signature

**Bad:**
```rust
/// Returns a String
pub fn get_name() -> String {
```

**Good:**
```rust
/// Returns the application name from the manifest
pub fn get_name() -> String {
```

### ❌ Don't Document Obvious Things

**Bad:**
```rust
/// The width
pub width: u32,
```

**Good:**
```rust
/// Window width in pixels (must be ≥ 200)
pub width: u32,
```

### ❌ Don't Use Vague Language

**Bad:**
- "This might fail..."
- "Usually returns..."
- "Probably should..."

**Good:**
- "Returns an error if..."
- "Returns `None` when..."
- "Must be called before..."

### ❌ Don't Document Implementation Details Users Don't Need

**Bad:**
```rust
/// Uses a BTreeMap internally for O(log n) lookup...
```

**Good:**
```rust
/// Stores configuration keys in sorted order
```

### ❌ Don't Forget Error Documentation

**Bad:**
```rust
pub fn parse(input: &str) -> Result<Config, Error> {
    // No error documentation
}
```

**Good:**
```rust
/// # Errors
///
/// Returns [`Error::InvalidFormat`] if input is not valid TOML.
pub fn parse(input: &str) -> Result<Config, Error> {
```

---

## Quality Checklist

Use this checklist when reviewing documentation:

### Module Level
- [ ] Module has `//!` documentation
- [ ] Purpose is clear in first sentence
- [ ] Architecture explained (with diagram if complex)
- [ ] Usage example provided
- [ ] Related modules cross-referenced

### Type Level (Structs/Enums)
- [ ] Purpose documented in first sentence
- [ ] All public fields documented
- [ ] Field constraints documented (ranges, invariants)
- [ ] Example showing construction/usage
- [ ] Related types cross-referenced

### Function Level
- [ ] Purpose clear in first sentence
- [ ] Parameters documented (if not obvious)
- [ ] Return value documented
- [ ] **All error cases documented**
- [ ] Panics documented (if any)
- [ ] Example provided for non-trivial functions
- [ ] Async behavior documented (if async)

### Examples
- [ ] Examples use correct attributes (`no_run`, etc.)
- [ ] Examples are complete (imports, error handling)
- [ ] Examples demonstrate intended usage
- [ ] Examples compile (unless using `ignore`)

### Cross-References
- [ ] Related functions linked
- [ ] Related types linked
- [ ] Error types linked in `# Errors` section

---

## Documentation Quality Levels

### Level 1: Minimal (Acceptable for Internal Code)
- One-line summary
- Basic parameter/return documentation

### Level 2: Standard (Required for Public APIs)
- One-line summary + context paragraph
- Parameter/return documentation
- Error documentation
- Example for non-trivial functions

### Level 3: Comprehensive (Target for Core Modules)
- Module-level documentation
- Complete function documentation
- Multiple examples
- Architecture diagrams
- Cross-references
- TypeScript usage examples (for ops)

### Level 4: Gold Standard (forge-weld, forge-etch, ext_registry)
- Everything in Level 3
- ASCII architecture diagrams
- Real-world usage patterns
- Common pitfalls documented
- Performance considerations
- Related resources linked

---

## Examples of Each Quality Level

### Minimal
```rust
/// Reads a file
pub fn read(path: &str) -> Result<String, Error> {
```

### Standard
```rust
/// Reads the entire contents of a file as a UTF-8 string
///
/// # Errors
///
/// Returns [`Error::NotFound`] if the file doesn't exist.
///
/// # Examples
///
/// ```no_run
/// let text = read("config.toml")?;
/// ```
pub fn read(path: &str) -> Result<String, Error> {
```

### Comprehensive
```rust
/// Reads the entire contents of a file as a UTF-8 string
///
/// Performs blocking I/O on the current thread. For async I/O,
/// use [`read_async()`] instead.
///
/// # Parameters
///
/// - `path`: File path (relative or absolute)
///
/// # Errors
///
/// - [`Error::NotFound`] - File doesn't exist
/// - [`Error::PermissionDenied`] - Insufficient permissions
/// - [`Error::InvalidUtf8`] - File contains invalid UTF-8
///
/// # Examples
///
/// ```no_run
/// use my_crate::fs::read;
///
/// let config = read("./config.toml")?;
/// println!("Config: {}", config);
/// ```
///
/// # See Also
///
/// - [`read_async()`] - Async version
/// - [`read_bytes()`] - Read as bytes without UTF-8 validation
pub fn read(path: &str) -> Result<String, Error> {
```

---

## Review Process

When reviewing documentation PRs, check for:

1. **Completeness**: All public items documented
2. **Clarity**: Can a new contributor understand it?
3. **Correctness**: Are error cases accurate?
4. **Consistency**: Follows this style guide
5. **Examples**: Complex functions have examples
6. **Links**: Related items cross-referenced

---

## See Also

- [DOCUMENTATION_TEMPLATES.md](./DOCUMENTATION_TEMPLATES.md) - Reusable templates
- [TYPE_MAPPING.md](./TYPE_MAPPING.md) - Rust ↔ TypeScript conversions
- [Rust Documentation Guidelines](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [cargo doc reference](https://doc.rust-lang.org/cargo/commands/cargo-doc.html)

---

## Quick Reference

| Aspect | Standard |
|--------|----------|
| **Voice** | Active, present tense |
| **Examples** | Use `no_run` for most |
| **Diagrams** | Box-drawing chars, 5-7 components max |
| **Errors** | Document all variants with recovery |
| **Links** | Liberal use of `` [`Type`] `` syntax |
| **Fields** | Document constraints and defaults |
| **Async** | Mention async behavior explicitly |
| **First line** | One-line summary (autocomplete) |
| **Target** | Level 3 (Comprehensive) for core modules |
