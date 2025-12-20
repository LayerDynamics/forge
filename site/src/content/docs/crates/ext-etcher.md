---
title: "ext_etcher"
description: Documentation generation extension providing the runtime:etcher module for building docs from code.
slug: crates/ext-etcher
---

The `ext_etcher` crate provides documentation generation capabilities for Forge applications through the `runtime:etcher` module.

## Overview

ext_etcher handles:

- **TypeScript Parsing** - Extract types, functions, and documentation from TypeScript
- **Rust Parsing** - Extract types and documentation from Rust source files
- **Markdown Generation** - Generate markdown documentation from source code
- **Site Generation** - Build static documentation sites (Astro/Starlight)
- **Symbol Extraction** - Resolve types and symbols across files

## Module: `runtime:etcher`

```typescript
import {
  parseTypeScript,
  parseRust,
  generateMarkdown,
  buildSite,
  resolveSymbol
} from "runtime:etcher";
```

## Key Types

### Configuration Types

```typescript
interface EtchConfig {
  // Source directories to parse
  sources: SourceConfig[];

  // Output directory for generated docs
  outputDir: string;

  // Site configuration
  site?: SiteConfig;

  // Markdown generation options
  markdown?: MarkdownConfig;
}

interface SourceConfig {
  // Path to source directory or file
  path: string;

  // Language: "typescript" | "rust"
  language: string;

  // Glob patterns to include
  include?: string[];

  // Glob patterns to exclude
  exclude?: string[];
}

interface SiteConfig {
  // Site title
  title: string;

  // Site description
  description?: string;

  // Base URL
  baseUrl?: string;

  // Theme: "starlight" | "docusaurus" | "custom"
  theme?: string;
}
```

### Parsed Types

```typescript
interface ParsedModule {
  // Module path
  path: string;

  // Exported functions
  functions: FunctionDoc[];

  // Exported types/interfaces
  types: TypeDoc[];

  // Exported classes
  classes: ClassDoc[];

  // Module-level documentation
  documentation?: string;
}

interface FunctionDoc {
  name: string;
  signature: string;
  parameters: ParameterDoc[];
  returnType: string;
  documentation?: string;
  examples?: string[];
  async: boolean;
}

interface TypeDoc {
  name: string;
  kind: "interface" | "type" | "enum";
  properties?: PropertyDoc[];
  documentation?: string;
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_etcher_parse_typescript` | `parseTypeScript(path)` | Parse TypeScript source file |
| `op_etcher_parse_rust` | `parseRust(path)` | Parse Rust source file |
| `op_etcher_generate_markdown` | `generateMarkdown(module)` | Generate markdown from parsed module |
| `op_etcher_build_site` | `buildSite(config)` | Build full documentation site |
| `op_etcher_resolve_symbol` | `resolveSymbol(name, context)` | Resolve type/symbol reference |

## Usage Example

```typescript
import { parseTypeScript, generateMarkdown, buildSite } from "runtime:etcher";

// Parse a TypeScript file
const module = await parseTypeScript("./src/api.ts");
console.log(`Found ${module.functions.length} functions`);

// Generate markdown for the module
const markdown = await generateMarkdown(module);
await Deno.writeTextFile("./docs/api.md", markdown);

// Build a full documentation site
await buildSite({
  sources: [
    { path: "./src", language: "typescript", include: ["**/*.ts"] },
    { path: "./crates", language: "rust", include: ["**/lib.rs"] }
  ],
  outputDir: "./site/docs",
  site: {
    title: "My Project Docs",
    theme: "starlight"
  }
});
```

## TypeScript Extraction

The TypeScript parser extracts:

- **Functions** - Name, parameters, return type, JSDoc
- **Interfaces** - Properties, methods, extends
- **Type Aliases** - Name, definition, generics
- **Classes** - Constructor, methods, properties
- **Enums** - Variants and values
- **Exports** - Re-exports and named exports

```typescript
/**
 * Greet a user by name.
 * @param name - The user's name
 * @returns A greeting message
 * @example
 * ```ts
 * greet("World") // "Hello, World!"
 * ```
 */
export function greet(name: string): string {
  return `Hello, ${name}!`;
}
```

## Rust Extraction

The Rust parser extracts:

- **Functions** - `pub fn` with doc comments
- **Structs** - `pub struct` with fields
- **Enums** - `pub enum` with variants
- **Traits** - `pub trait` with methods
- **Type Aliases** - `pub type`
- **Modules** - `pub mod` structure

```rust
/// Calculate the factorial of a number.
///
/// # Arguments
/// * `n` - The number to calculate factorial for
///
/// # Returns
/// The factorial of n
///
/// # Examples
/// ```
/// assert_eq!(factorial(5), 120);
/// ```
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}
```

## Generated Markdown Format

```markdown
# Module: api

Brief module description from first doc comment.

## Functions

### greet(name)

Greet a user by name.

**Parameters:**
- `name` (string) - The user's name

**Returns:** string - A greeting message

**Example:**
```ts
greet("World") // "Hello, World!"
```
```

## File Structure

```text
crates/ext_etcher/
├── src/
│   ├── lib.rs        # Extension implementation
│   ├── typescript.rs # TypeScript parser
│   ├── rust.rs       # Rust parser
│   ├── markdown.rs   # Markdown generator
│   └── site.rs       # Site builder
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `swc_ecma_parser` | TypeScript/JavaScript parsing |
| `syn` | Rust source parsing |
| `pulldown-cmark` | Markdown processing |

## Related

- [forge-etch](/docs/crates/forge-etch) - Core documentation generation library
- [forge-weld](/docs/crates/forge-weld) - SDK code generation
