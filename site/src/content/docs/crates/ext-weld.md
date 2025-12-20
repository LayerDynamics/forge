---
title: "ext_weld"
description: Code generation extension providing the forge:weld module.
slug: crates/ext-weld
---

The `ext_weld` crate exposes forge-weld code generation capabilities as a runtime module for Forge applications through the `forge:weld` module.

## Overview

ext_weld provides runtime access to:

- **TypeScript transpilation** - Convert TypeScript to JavaScript at runtime
- **Type generation** - Generate TypeScript interfaces from definitions
- **Module registration** - Register and generate SDK modules dynamically
- **Code validation** - Validate TypeScript syntax

## Module: `forge:weld`

```typescript
import {
  info,
  transpile,
  generateDts,
  jsonToInterface,
  validateTs,
  registerModule,
  listModules,
  generateModule,
  generateFromDefinition
} from "forge:weld";
```

## Key Types

### Extension Info

```typescript
interface ExtensionInfo {
  name: string;
  version: string;
  capabilities: string[];
}
```

### Transpilation

```typescript
interface TranspileOptions {
  filename?: string;     // Source file name (for error messages)
  sourceMap?: boolean;   // Whether to include source maps
  minify?: boolean;      // Whether to minify output
}

interface TranspileResult {
  code: string;          // Transpiled JavaScript code
  sourceMap?: string;    // Source map (if requested)
}
```

### Validation

```typescript
interface ValidationResult {
  valid: boolean;
  errors: string[];
}
```

### Type Definitions

```typescript
interface TypeDefinition {
  name: string;          // Type name
  definition: string;    // TypeScript type definition
}
```

### Module Definition

```typescript
interface RuntimeModuleDefinition {
  name: string;          // Module name (e.g., "my_module")
  specifier: string;     // Module specifier (e.g., "custom:my-module")
  doc?: string;          // Documentation for the module
  structs: RuntimeStructDefinition[];
  enums: RuntimeEnumDefinition[];
  ops: RuntimeOpDefinition[];
}

interface RuntimeStructDefinition {
  name: string;
  tsName?: string;
  doc?: string;
  fields: RuntimeFieldDefinition[];
}

interface RuntimeFieldDefinition {
  name: string;
  tsName?: string;
  tsType: string;
  doc?: string;
  optional?: boolean;
  readonly?: boolean;
}

interface RuntimeEnumDefinition {
  name: string;
  tsName?: string;
  doc?: string;
  variants: RuntimeVariantDefinition[];
}

interface RuntimeVariantDefinition {
  name: string;
  value?: string;
  doc?: string;
  dataType?: string;
}

interface RuntimeOpDefinition {
  rustName: string;
  tsName?: string;
  doc?: string;
  isAsync?: boolean;
  params: RuntimeParamDefinition[];
  returnType?: string;
}

interface RuntimeParamDefinition {
  name: string;
  tsName?: string;
  tsType: string;
  doc?: string;
  optional?: boolean;
}
```

### Generated Code

```typescript
interface GeneratedCode {
  code: string;   // Generated TypeScript/JavaScript code
  dts: string;    // Generated .d.ts declarations
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_weld_info` | `info()` | Get extension information |
| `op_weld_transpile` | `transpile(source, opts?)` | Transpile TypeScript to JavaScript |
| `op_weld_generate_dts` | `generateDts(types)` | Generate .d.ts from type definitions |
| `op_weld_json_to_interface` | `jsonToInterface(name, json)` | Generate interface from JSON schema |
| `op_weld_validate_ts` | `validateTs(source)` | Validate TypeScript syntax |
| `op_weld_register_module` | `registerModule(def)` | Register a module for generation |
| `op_weld_list_modules` | `listModules()` | List all registered modules |
| `op_weld_generate_module_ts` | `generateModuleTs(specifier)` | Generate TypeScript for module |
| `op_weld_generate_module_dts` | `generateModuleDts(specifier)` | Generate .d.ts for module |
| `op_weld_generate_module` | `generateModule(specifier)` | Generate both TS and .d.ts |
| `op_weld_generate_from_definition` | `generateFromDefinition(def)` | Generate code from inline definition |

## Usage Examples

### Transpile TypeScript at Runtime

```typescript
import { transpile, validateTs } from "forge:weld";

// Validate syntax first
const validation = validateTs(`
  const greet = (name: string): string => {
    return \`Hello, \${name}!\`;
  };
`);

if (validation.valid) {
  const result = transpile(`
    const greet = (name: string): string => {
      return \`Hello, \${name}!\`;
    };
  `, { filename: "greeting.ts" });
  console.log(result.code);
}
```

### Generate Interface from JSON

```typescript
import { jsonToInterface } from "forge:weld";

const jsonSchema = JSON.stringify({
  name: "string",
  age: 25,
  isActive: true
});

const interface_ = jsonToInterface("User", jsonSchema);
// Generates:
// export interface User {
//   name: string;
//   age: number;
//   isActive: boolean;
// }
```

### Register and Generate Module

```typescript
import { registerModule, generateModule } from "forge:weld";

// Define a custom module
registerModule({
  name: "my_api",
  specifier: "custom:my-api",
  doc: "My custom API module",
  structs: [{
    name: "ApiResponse",
    fields: [
      { name: "data", tsType: "unknown" },
      { name: "status", tsType: "number" }
    ]
  }],
  enums: [],
  ops: [{
    rustName: "op_api_fetch",
    tsName: "fetch",
    isAsync: true,
    params: [{ name: "url", tsType: "string" }],
    returnType: "ApiResponse"
  }]
});

// Generate TypeScript SDK
const generated = generateModule("custom:my-api");
console.log(generated.code);
console.log(generated.dts);
```

## File Structure

```text
crates/ext_weld/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `forge-weld` | Code generation utilities |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `serde`, `serde_json` | JSON handling |
| `thiserror` | Error types |
| `deno_error` | JavaScript error conversion |
| `tracing` | Logging |
| `linkme` | Compile-time symbol collection |

## Related

- [forge-weld](/docs/crates/forge-weld) - Core code generation library
- [forge-weld-macro](/docs/crates/forge-weld-macro) - Procedural macros
- [forge:bundler](/docs/crates/ext-bundler) - Related forge module
