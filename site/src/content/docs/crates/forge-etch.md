---
title: "forge-etch"
description: "Documentation generator for Forge framework extensions"
slug: docs/crates/forge-etch
---

Documentation generator for Forge framework extensions. Parses TypeScript sources and forge-weld IR to generate comprehensive API documentation.

## Overview

`forge-etch` generates documentation from Forge extensions by:

1. **Parsing TypeScript** - Uses deno_ast/SWC to parse `ts/init.ts` files
2. **Extracting Metadata** - Reads forge-weld IR (ops, structs, enums)
3. **Merging Sources** - Combines docs with TypeScript JSDoc taking precedence
4. **Generating Output** - Creates Astro-compatible markdown or standalone HTML

## Architecture

```
┌─────────────────┐    ┌──────────────────┐
│ ts/init.ts      │    │ forge-weld IR    │
│ (SWC parse)     │    │ (ops/structs)    │
└────────┬────────┘    └────────┬─────────┘
         │                      │
         └──────────┬───────────┘
                    ▼
             ┌──────────────┐
             │   EtchNode   │
             └──────┬───────┘
                    │
         ┌──────────┴──────────┐
         ▼                     ▼
   ┌──────────┐         ┌──────────┐
   │ Astro MD │         │   HTML   │
   └──────────┘         └──────────┘
```

## Installation

Add to your `Cargo.toml`:

```toml
[build-dependencies]
forge-etch = { path = "../forge-etch" }
```

## Quick Start

### Using EtchBuilder in build.rs

```rust
use forge_etch::EtchBuilder;

fn main() {
    EtchBuilder::new("host_fs", "runtime:fs")
        .rust_source("src/lib.rs")
        .ts_source("ts/init.ts")
        .output_dir("docs")
        .generate_astro(true)
        .generate_html(true)
        .build()
        .expect("Failed to generate docs");
}
```

### Using Etcher Directly

```rust
use forge_etch::{Etcher, EtchConfig};

fn main() {
    let config = EtchConfig::new("fs", "runtime:fs");
    let mut etcher = Etcher::new(config);

    // Run the documentation pipeline
    let output = etcher.run().unwrap();

    println!("Generated {} files", output.all_files().count());
    println!("Documented {} symbols", output.symbol_count);

    // Print terminal preview
    etcher.print_preview();
}
```

## API Reference

### EtchBuilder

Builder pattern API for configuring documentation generation.

```rust
pub struct EtchBuilder {
    pub name: String,              // Extension name (e.g., "host_fs")
    pub module_specifier: String,  // Module specifier (e.g., "runtime:fs")
    pub rust_source: Option<PathBuf>,
    pub ts_source: Option<PathBuf>,
    pub output_dir: PathBuf,
    pub generate_astro: bool,      // Default: true
    pub generate_html: bool,       // Default: false
    pub title: Option<String>,
    pub description: Option<String>,
    pub include_private: bool,     // Include private symbols
    pub include_internal: bool,    // Include @internal symbols
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new(name, module_specifier)` | Create new builder |
| `rust_source(path)` | Set Rust source file |
| `ts_source(path)` | Set TypeScript source file |
| `output_dir(path)` | Set output directory (default: "docs") |
| `generate_astro(bool)` | Enable Astro markdown generation |
| `generate_html(bool)` | Enable HTML generation |
| `title(string)` | Set documentation title |
| `description(string)` | Set documentation description |
| `include_private(bool)` | Include private symbols |
| `include_internal(bool)` | Include @internal symbols |
| `add_source_dir(path)` | Add source directory to scan |
| `build()` | Run documentation generation |
| `from_crate_root(name, specifier, root)` | Auto-discover sources |

#### Example

```rust
use forge_etch::EtchBuilder;

let output = EtchBuilder::new("host_fs", "runtime:fs")
    .rust_source("src/lib.rs")
    .ts_source("ts/init.ts")
    .output_dir("generated/docs")
    .generate_astro(true)
    .generate_html(true)
    .title("File System API")
    .description("Read, write, and manage files")
    .include_private(false)
    .build()?;

println!("Generated {} Astro files", output.astro_files.len());
println!("Generated {} HTML files", output.html_files.len());
```

### EtchConfig

Configuration structure for the Etcher.

```rust
pub struct EtchConfig {
    pub name: String,
    pub module_specifier: String,
    pub rust_source: Option<PathBuf>,
    pub ts_source: Option<PathBuf>,
    pub output_dir: PathBuf,
    pub generate_astro: bool,
    pub generate_html: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub include_private: bool,
    pub include_internal: bool,
}
```

### Etcher

The main documentation generator that coordinates the entire pipeline.

```rust
pub struct Etcher {
    config: EtchConfig,
    diagnostics: DiagnosticsCollector,
    nodes: Vec<EtchNode>,
    weld_module: Option<WeldModule>,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new(config)` | Create new Etcher |
| `with_weld_module(module)` | Set WeldModule for Rust integration |
| `config()` | Get configuration reference |
| `nodes()` | Get extracted nodes |
| `diagnostics()` | Get diagnostics collector |
| `run()` | Run the documentation pipeline |
| `preview()` | Generate terminal preview string |
| `print_preview()` | Print preview to stdout with colors |
| `preview_plain()` | Generate preview without colors |

### BuildOutput

Result of documentation generation.

```rust
pub struct BuildOutput {
    pub astro_files: Vec<PathBuf>,   // Generated Astro markdown files
    pub html_files: Vec<PathBuf>,    // Generated HTML files
    pub output_dir: PathBuf,         // Output directory
    pub symbol_count: usize,         // Number of documented symbols
}
```

### EtchNode

Represents a documented symbol (function, interface, class, etc.).

```rust
pub struct EtchNode {
    pub name: String,
    pub def: EtchNodeDef,
    pub doc: EtchDoc,
    pub visibility: Visibility,
    pub location: Option<Location>,
}

pub enum EtchNodeDef {
    Function(FunctionDef),
    Interface(InterfaceDef),
    Class(ClassDef),
    TypeAlias(TypeAliasDef),
    Enum(EnumDef),
    Variable(VariableDef),
    Op(OpDef),
    ModuleDoc,
}
```

### EtchDoc

Documentation extracted from JSDoc comments.

```rust
pub struct EtchDoc {
    pub description: Option<String>,
    pub tags: Vec<JsDocTag>,
}

pub enum JsDocTag {
    Param { name: String, ty: Option<String>, description: Option<String> },
    Returns { ty: Option<String>, description: Option<String> },
    Example { code: String, language: Option<String> },
    Throws { ty: Option<String>, description: Option<String> },
    Deprecated { description: Option<String> },
    See { reference: String },
    Since { version: String },
    Internal,
    Private,
    Public,
    // ... more tags
}
```

## Output Generators

### AstroGenerator

Generates Astro-compatible markdown for documentation sites.

```rust
use forge_etch::AstroGenerator;

let generator = AstroGenerator::new("docs/api".into());
let files = generator.generate(&extension_doc)?;
```

**Features:**
- Astro frontmatter with title, description, slug
- Starlight sidebar integration
- Compatible with Astro versions 2.x and 3.x
- Table of contents support
- Code syntax highlighting

### HtmlGenerator

Generates standalone HTML documentation.

```rust
use forge_etch::HtmlGenerator;

let generator = HtmlGenerator::new("docs/html".into())?;
let files = generator.generate(&extension_doc)?;
```

**Features:**
- Self-contained HTML files
- Embedded CSS styling
- Search functionality
- Copy-to-clipboard for code blocks
- Dark mode support

## Astro Integration

### Configuration Validation

```rust
use forge_etch::{check_config, AstroConfig};

let result = check_config("path/to/astro.config.mjs")?;
if result.is_valid {
    println!("Astro configuration is valid");
} else {
    for error in result.errors {
        eprintln!("Error: {}", error);
    }
}
```

### Version Compatibility

```rust
use forge_etch::{detect_version, supports_feature, AstroVersion};

let version = detect_version("path/to/project")?;
match version {
    AstroVersion::V2 => println!("Astro 2.x"),
    AstroVersion::V3 => println!("Astro 3.x"),
    AstroVersion::V4 => println!("Astro 4.x"),
}

if supports_feature(&version, "content_collections") {
    // Use content collections
}
```

### Slug Generation

```rust
use forge_etch::{astro_slug, file_slug, anchor_slug, unique_slug};

let slug = astro_slug("My Function Name");      // "my-function-name"
let file = file_slug("runtime:fs");             // "runtime-fs"
let anchor = anchor_slug("readText(path)");     // "readtext-path"
let unique = unique_slug("item", &existing);    // "item-2" if "item" exists
```

## TypeScript Parsing

### Extracting Exports

```rust
use forge_etch::{extract_exports, is_typescript_file, is_declaration_file};

if is_typescript_file(&path) {
    let exports = extract_exports(&path)?;
    for export in exports {
        println!("Export: {} ({})", export.name, export.kind);
    }
}

if is_declaration_file(&path) {
    println!("This is a .d.ts file");
}
```

## Deno Integration

### Configuration

```rust
use forge_etch::{DenoConfig, deno_version, is_deno_runtime};

// Check if running in Deno
if is_deno_runtime() {
    println!("Deno version: {}", deno_version()?);
}

// Parse deno.json
let config = DenoConfig::from_path("deno.json")?;
println!("Tasks: {:?}", config.tasks);
```

### Module Imports

```rust
use forge_etch::{jsr_import, jsr_import_latest, generate_deno_imports, ModuleImport};

// Generate JSR import
let import = jsr_import("std", "path", "1.0.0");
// "@jsr/std__path@1.0.0"

let latest = jsr_import_latest("std", "path");
// "@jsr/std__path"

// Generate multiple imports
let imports = generate_deno_imports(&[
    ModuleImport { scope: "std", name: "path", version: Some("1.0.0") },
    ModuleImport { scope: "std", name: "fs", version: None },
]);
```

## Asset Embedding

### Standalone HTML

```rust
use forge_etch::{generate_standalone_html, EmbedConfig};

let config = EmbedConfig {
    include_css: true,
    include_js: true,
    minify: true,
    dark_mode: true,
};

let html = generate_standalone_html(&content, &config)?;
```

### Built-in Assets

```rust
use forge_etch::{get_asset, list_assets, DEFAULT_CSS, SEARCH_JS, COPY_BUTTON_JS};

// Get default CSS
println!("{}", DEFAULT_CSS);

// List all embedded assets
for asset in list_assets() {
    println!("{}: {} bytes", asset.name, asset.content.len());
}

// Get specific asset
if let Some(asset) = get_asset("search.js") {
    println!("Search JS: {}", asset.content);
}
```

## Terminal Printing

### EtchPrinter

```rust
use forge_etch::EtchPrinter;

let printer = EtchPrinter::new(&nodes, true, false);
printer.print_to_stdout();  // With colors

// Without colors
let plain = EtchPrinter::new(&nodes, false, false);
println!("{}", plain);
```

## Diagnostics

### Error Handling

```rust
use forge_etch::{EtchError, EtchResult};

fn process_docs() -> EtchResult<()> {
    let builder = EtchBuilder::new("fs", "runtime:fs");

    match builder.build() {
        Ok(output) => {
            println!("Success: {} symbols", output.symbol_count);
            Ok(())
        }
        Err(EtchError::Parse(msg)) => {
            eprintln!("Parse error: {}", msg);
            Err(EtchError::Parse(msg))
        }
        Err(EtchError::Io(e)) => {
            eprintln!("IO error: {}", e);
            Err(EtchError::Io(e))
        }
        Err(e) => Err(e),
    }
}
```

## Module Structure

| Module | Description |
|--------|-------------|
| `builder` | EtchBuilder API for configuration |
| `docgen` | Documentation generation orchestration |
| `parser` | TypeScript parsing with SWC |
| `astro` | Astro markdown generation |
| `html` | Standalone HTML generation |
| `node` | EtchNode representation |
| `js_doc` | JSDoc comment parsing |
| `types` | Type system representations |
| `embed` | Asset embedding utilities |
| `deno` | Deno runtime utilities |
| `printer` | Terminal output formatting |
| `diagnostics` | Error handling and reporting |

## Type System

### EtchType

```rust
pub struct EtchType {
    pub kind: EtchTypeKind,
}

pub enum EtchTypeKind {
    Primitive(EtchPrimitive),
    Literal(EtchLiteral),
    Reference { name: String, type_params: Vec<EtchType> },
    Array { element: Box<EtchType> },
    Tuple { elements: Vec<EtchType> },
    Object { properties: Vec<PropertyDef> },
    Function { params: Vec<ParamDef>, return_type: Box<EtchType> },
    Union { members: Vec<EtchType> },
    Intersection { members: Vec<EtchType> },
    Generic { name: String, constraint: Option<Box<EtchType>> },
    Mapped { // ... },
    Conditional { // ... },
}

pub enum EtchPrimitive {
    String,
    Number,
    Boolean,
    Null,
    Undefined,
    Void,
    Any,
    Unknown,
    Never,
    BigInt,
    Symbol,
}
```

## CLI Usage

The `forge docs` command uses forge-etch internally:

```bash
# Generate docs for all extensions
forge docs --all-extensions

# Generate docs for specific extension
forge docs -e fs -o docs/api

# Generate HTML output
forge docs -e fs --html

# Include private symbols
forge docs -e fs --include-private
```

## Best Practices

### Documentation Quality

1. **Write JSDoc Comments** - JSDoc from TypeScript takes precedence over generated docs
2. **Include Examples** - Use `@example` tags with runnable code
3. **Document Parameters** - Use `@param` tags with type annotations
4. **Document Returns** - Use `@returns` tags with descriptions
5. **Mark Internal APIs** - Use `@internal` for implementation details

### Build Integration

```rust
// build.rs
fn main() {
    // Only generate docs in release mode
    if std::env::var("PROFILE").unwrap_or_default() == "release" {
        if let Err(e) = EtchBuilder::new("fs", "runtime:fs")
            .ts_source("ts/init.ts")
            .output_dir("docs")
            .build()
        {
            eprintln!("Warning: Doc generation failed: {}", e);
            // Don't fail the build for doc issues
        }
    }
}
```

### Testing Documentation

```rust
#[cfg(test)]
mod tests {
    use forge_etch::{EtchBuilder, EtchConfig, Etcher};

    #[test]
    fn test_docs_generate() {
        let output = EtchBuilder::new("test", "runtime:test")
            .ts_source("tests/fixtures/test.ts")
            .output_dir("target/test-docs")
            .build()
            .expect("Should generate docs");

        assert!(output.symbol_count > 0);
        assert!(!output.astro_files.is_empty());
    }
}
```
