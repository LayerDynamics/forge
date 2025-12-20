# Forge Documentation Templates

Reusable templates for documenting Forge codebase. These templates are based on gold-standard examples from existing modules.

## Table of Contents

1. [Module-Level Documentation](#1-module-level-documentation)
2. [Struct Documentation](#2-struct-documentation)
3. [Function Documentation](#3-function-documentation)
4. [Error Documentation](#4-error-documentation)
5. [Builder Pattern Documentation](#5-builder-pattern-documentation)
6. [Enum Documentation](#6-enum-documentation)
7. [Extension Crate Documentation](#7-extension-crate-documentation)

---

## 1. Module-Level Documentation

Module documentation should appear at the top of the file using `//!` comments.

### Template

```rust
//! [Module Name] - [Brief one-line description]
//!
//! [2-3 sentence overview of what this module does and why it exists]
//!
//! # Architecture
//!
//! [Optional ASCII diagram showing component relationships]
//!
//! ```text
//! ┌─────────────────────┐
//! │   Component A       │
//! └──────────┬──────────┘
//!            │
//!    ┌───────▼────────┐
//!    │  Component B   │
//!    └────────────────┘
//! ```
//!
//! # Key Concepts
//!
//! - **Concept 1**: Explanation
//! - **Concept 2**: Explanation
//! - **Concept 3**: Explanation
//!
//! # Usage
//!
//! ```no_run
//! use crate::module_name::Type;
//!
//! let example = Type::new();
//! example.do_something()?;
//! ```
//!
//! # Implementation Details
//!
//! [Describe important implementation details, constraints, or design decisions]
//!
//! # Examples
//!
//! See `examples/` directory:
//! - `examples/basic-usage` - Basic usage patterns
//! - `examples/advanced` - Advanced scenarios
```

### Real-World Example

From `crates/forge-runtime/src/ext_registry.rs`:

```rust
//! Extension Registry - Centralized management of all Forge runtime extensions
//!
//! This module provides:
//! - `ExtensionRegistry`: Central registry for all ext_* extensions
//! - `ExtensionDescriptor`: Metadata about each extension
//! - `ExtensionInitContext`: Context for state initialization
//!
//! All extensions are available at both build time (for TypeScript binding generation)
//! and runtime (for app execution). Users import only what they need via `runtime:*` specifiers.
```

---

## 2. Struct Documentation

Structs should document their purpose, fields, and any invariants.

### Template

```rust
/// [Brief one-line description of what this struct represents]
///
/// [2-3 sentences explaining the struct's role, when it's used, and any key characteristics]
///
/// # Fields
///
/// - `field_name`: [Explanation of what this field represents and constraints]
///
/// # Invariants
///
/// [Optional: Any invariants that must hold, e.g., "width must be > 0"]
///
/// # Examples
///
/// ```no_run
/// let instance = MyStruct {
///     field: value,
/// };
/// ```
#[derive(Debug)]
pub struct MyStruct {
    /// [Field-level doc: What this field stores]
    pub field_name: Type,

    /// [Private fields should also be documented]
    internal_state: OtherType,
}
```

### Real-World Example

From `crates/forge-runtime/src/main.rs`:

```rust
/// Application manifest (manifest.app.toml)
///
/// Defines app metadata, window configuration, and permissions.
/// Loaded at runtime startup.
#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    /// App metadata (name, version, identifier)
    pub app: App,
    /// Default window configuration (optional)
    pub windows: Option<Windows>,
    /// Permissions/capabilities section. Accepts both `permissions` and `capabilities` keys.
    #[serde(alias = "capabilities")]
    pub permissions: Option<Permissions>,
}
```

---

## 3. Function Documentation

Functions should document purpose, parameters, return values, errors, and examples.

### Template

```rust
/// [One-line description of what this function does]
///
/// [2-3 sentences providing context: when to use this, what it accomplishes,
/// any important side effects or state changes]
///
/// # Parameters
///
/// - `param_name`: [What this parameter represents, valid values/ranges]
///
/// # Returns
///
/// [Description of return value, including what `None`/`Err` means if applicable]
///
/// # Errors
///
/// This function returns an error in the following cases:
///
/// - [`ErrorType::Variant`] - When [condition] occurs
///   - Example: [concrete example]
///   - Recovery: [how to fix]
///
/// # Panics
///
/// [Optional: Document any panics, or state "This function does not panic"]
///
/// # Examples
///
/// ```no_run
/// let result = my_function(arg1, arg2)?;
/// assert_eq!(result, expected);
/// ```
///
/// # See Also
///
/// - [`RelatedFunction`] - Related functionality
pub fn my_function(param: Type) -> Result<ReturnType, Error> {
    // ...
}
```

### Real-World Example

From `crates/forge_cli/src/main.rs`:

```rust
/// Locate the forge-runtime binary
///
/// Searches multiple locations in order of preference:
/// 1. Custom path from FORGE_RUNTIME_PATH env var
/// 2. Workspace target directory (cargo build output)
/// 3. System PATH
///
/// This function is used by all commands that need to spawn forge-runtime.
///
/// # Errors
///
/// This function returns an error in the following cases:
///
/// - [`io::Error`] - When binary not found in any location
///   - Example: No forge-runtime in PATH, not built in workspace
///   - Recovery: Run `cargo build -p forge-runtime` or set FORGE_RUNTIME_PATH
///
/// # Examples
///
/// ```no_run
/// let runtime_path = find_forge_host()?;
/// println!("Found runtime at: {}", runtime_path.display());
/// ```
fn find_forge_host() -> Result<PathBuf, io::Error> {
    // ...
}
```

---

## 4. Error Documentation

Error documentation should use the `# Errors` section to explain all failure modes.

### Template

```rust
/// # Errors
///
/// This function returns an error in the following cases:
///
/// - [`ErrorType::Variant1`]
///   - **When**: [Condition that triggers this error]
///   - **Example**: [Concrete code example or scenario]
///   - **Recovery**: [How to fix or work around this error]
///
/// - [`ErrorType::Variant2`]
///   - **When**: [Condition that triggers this error]
///   - **Example**: [Concrete code example or scenario]
///   - **Recovery**: [How to fix or work around this error]
///
/// - [`ErrorType::Variant3`]
///   - **When**: [Condition that triggers this error]
///   - **Example**: [Concrete code example or scenario]
///   - **Recovery**: [How to fix or work around this error]
```

### Real-World Example

From `crates/forge-weld/src/build/extension.rs`:

```rust
/// # Errors
///
/// This function returns an error in the following cases:
///
/// - [`ExtensionBuilderError::EnvVarMissing`]
///   - When `OUT_DIR` or `CARGO_MANIFEST_DIR` not set (non-cargo build)
///   - **Recovery:** Only call from build.rs scripts run by cargo
///
/// - [`ExtensionBuilderError::TsNotFound`]
///   - When TypeScript source file doesn't exist at specified path
///   - Example: `ts_path("ts/init.ts")` but file is missing
///   - **Recovery:** Verify file path is correct relative to crate root
///
/// - [`ExtensionBuilderError::TranspileError`]
///   - When Deno fails to transpile TypeScript to JavaScript
///   - Causes: Syntax errors, type errors (if strict mode), import resolution
///   - **Recovery:** Check TypeScript source for errors, verify imports
```

---

## 5. Builder Pattern Documentation

Builder patterns should document the build flow and all builder methods.

### Template

```rust
/// Builder for [what is being built]
///
/// Provides a fluent API for configuring [thing] before construction.
///
/// # Build Flow
///
/// 1. Create builder with [`Self::new()`]
/// 2. Configure with builder methods ([`Self::method1()`], [`Self::method2()`])
/// 3. Finalize with [`Self::build()`]
///
/// # Required Configuration
///
/// - [Required method 1]
/// - [Required method 2]
///
/// # Optional Configuration
///
/// - [Optional method 1] - Defaults to [value]
/// - [Optional method 2] - Defaults to [value]
///
/// # Examples
///
/// ```no_run
/// let instance = MyBuilder::new("name")
///     .option1(value1)
///     .option2(value2)
///     .build()?;
/// ```
pub struct MyBuilder {
    // fields
}

impl MyBuilder {
    /// Create a new builder with required parameters
    ///
    /// # Parameters
    ///
    /// - `required_param`: [Description]
    pub fn new(required_param: Type) -> Self {
        // ...
    }

    /// Configure [aspect]
    ///
    /// [Explain what this configures and its effect on the built object]
    ///
    /// # Default
    ///
    /// If not called, defaults to [value].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// builder.method_name(value);
    /// ```
    pub fn method_name(mut self, param: Type) -> Self {
        // ...
        self
    }

    /// Build the final instance
    ///
    /// Consumes the builder and constructs the configured object.
    ///
    /// # Errors
    ///
    /// [Document all errors - see Error Documentation template]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let result = builder.build()?;
    /// ```
    pub fn build(self) -> Result<Target, Error> {
        // ...
    }
}
```

### Real-World Example

From `crates/forge-weld/src/build/extension.rs`:

```rust
/// Builder for Forge extension crates
///
/// Simplifies build.rs scripts by handling:
/// - TypeScript transpilation
/// - extension.rs generation
/// - .d.ts generation for SDK
/// - cargo:rerun-if-changed directives
///
/// # Example
/// ```no_run
/// use forge_weld::ExtensionBuilder;
///
/// fn main() {
///     ExtensionBuilder::new("host_fs", "runtime:fs")
///         .ts_path("ts/init.ts")
///         .ops(&[
///             "op_fs_read_text",
///             "op_fs_write_text",
///         ])
///         .generate_sdk_types("../../sdk")
///         .build()
///         .expect("Failed to build extension");
/// }
/// ```
pub struct ExtensionBuilder {
    // ...
}
```

---

## 6. Enum Documentation

Enums should document each variant and when it's used.

### Template

```rust
/// [One-line description of what this enum represents]
///
/// [Explain the purpose of this enum and when each variant is used]
///
/// # Variants
///
/// - [`Self::Variant1`]: [When this variant is used]
/// - [`Self::Variant2`]: [When this variant is used]
///
/// # Examples
///
/// ```no_run
/// match status {
///     Status::Variant1 => { /* ... */ },
///     Status::Variant2(data) => { /* ... */ },
/// }
/// ```
#[derive(Debug)]
pub enum MyEnum {
    /// [Description of this variant and when it occurs]
    Variant1,

    /// [Description of this variant and its data]
    Variant2 {
        /// [What this field contains]
        field: Type,
    },
}
```

### Real-World Example

From `crates/forge-runtime/src/ext_registry.rs`:

```rust
/// Extension initialization complexity tier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionTier {
    /// No state initialization required - just register the extension
    ExtensionOnly,
    /// Simple state initialization (no external dependencies)
    SimpleState,
    /// Requires capability adapter injection
    CapabilityBased,
    /// Requires complex runtime context (channels, app info, etc.)
    ComplexContext,
}
```

---

## 7. Extension Crate Documentation

Extension crates (`ext_*`) should follow a standard structure.

### Template: `src/lib.rs`

```rust
//! [Extension Name] - [One-line description]
//!
//! Provides the `runtime:[name]` module for Forge apps.
//!
//! # API Overview
//!
//! - **[Category 1]**: [`function1`], [`function2`]
//! - **[Category 2]**: [`function3`], [`function4`]
//!
//! # TypeScript Usage
//!
//! ```typescript
//! import * as moduleName from "runtime:module_name";
//!
//! const result = await moduleName.functionName(args);
//! ```
//!
//! # Permissions
//!
//! This extension requires the following permissions:
//!
//! ```toml
//! [permissions.category]
//! operation = ["pattern"]
//! ```
//!
//! # Examples
//!
//! ## [Use Case 1]
//! ```no_run
//! #[weld_op(async)]
//! #[op2(async)]
//! async fn op_example() -> Result<String, Error> {
//!     // ...
//! }
//! ```

use deno_core::op2;
use forge_weld_macro::weld_op;

// Extension ops here
```

### Template: `build.rs`

```rust
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("[ext_name]", "runtime:[name]")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_function1",
            "op_function2",
        ])
        .generate_sdk_module("../../sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build [ext_name] extension");
}
```

### Template: `ts/init.ts`

```typescript
/**
 * [Extension Name] - [One-line description]
 *
 * @module runtime:[name]
 */

// Import core ops
const core = Deno.core;
const ops = core.ops;

/**
 * [Function description]
 *
 * @param param - [Parameter description]
 * @returns [Return value description]
 *
 * @example
 * ```typescript
 * const result = await functionName(arg);
 * ```
 */
export function functionName(param: Type): ReturnType {
    return ops.op_function_name(param);
}
```

---

## Best Practices

### 1. Documentation Completeness Checklist

For every module, ensure:
- [ ] Module-level documentation with purpose and architecture
- [ ] All public types documented
- [ ] All public functions documented
- [ ] Error cases documented with recovery steps
- [ ] Examples provided for complex functionality
- [ ] Cross-references to related types/functions

### 2. When to Use ASCII Diagrams

Use ASCII diagrams for:
- **Architecture overviews**: Component relationships
- **Data flow**: IPC, event loops, pipelines
- **State machines**: Extension initialization tiers
- **Directory structures**: Build outputs, app layouts

Keep diagrams simple and focused on one concept.

### 3. Example Code Guidelines

- Use `no_run` for code that requires external setup or cannot run in doctests
- Use `ignore` only when absolutely necessary (syntax highlighting without compilation)
- Prefer complete, runnable examples over fragments
- Show error handling in examples (use `?` or `unwrap()` explicitly)

### 4. Cross-Referencing

Use Rust's linking syntax liberally:
- `` [`Type`] `` - Link to type in same module
- `` [`module::Type`] `` - Link to type in another module
- `` [`Self::method()`] `` - Link to method on current type

### 5. Field-Level Documentation

Document fields when:
- Field purpose isn't obvious from name
- Field has constraints or valid ranges
- Field affects behavior in non-obvious ways

```rust
/// Window width in pixels
///
/// Must be between 200 and screen width. Defaults to 800.
pub width: u32,
```

---

## Templates Summary

**Quick Reference:**

| Template | Use For | Key Sections |
|----------|---------|--------------|
| Module-Level | Top of `.rs` files | Architecture, Usage, Examples |
| Struct | Type definitions | Purpose, Fields, Invariants |
| Function | Operations | Parameters, Returns, Errors, Examples |
| Error | Result-returning fns | All error variants with recovery |
| Builder | Builder patterns | Build flow, Methods, Build errors |
| Enum | Enum types | Variant meanings, Match examples |
| Extension | ext_* crates | API overview, Permissions, TypeScript usage |

---

## See Also

- [DOCUMENTATION_STYLE_GUIDE.md](./DOCUMENTATION_STYLE_GUIDE.md) - Consistency standards
- [TYPE_MAPPING.md](./TYPE_MAPPING.md) - Rust ↔ TypeScript type conversions
- [forge-weld/src/lib.rs](../crates/forge-weld/src/lib.rs) - Gold standard module docs
- [forge-runtime/src/ext_registry.rs](../crates/forge-runtime/src/ext_registry.rs) - Excellent architecture docs
