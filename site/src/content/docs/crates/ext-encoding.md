---
title: "ext_encoding"
description: TextEncoder/TextDecoder extension providing Web-standard encoding APIs.
slug: crates/ext-encoding
---

The `ext_encoding` crate provides standard Web TextEncoder and TextDecoder APIs for Forge applications through the `runtime:encoding` module.

## Overview

ext_encoding provides:

- **TextEncoder** - Encode strings to UTF-8 byte arrays
- **TextDecoder** - Decode byte arrays back to strings
- **Web Compatibility** - Standard WHATWG Encoding API implementation
- **Pure JavaScript** - No Rust ops required, efficient JS implementation

> **Note:** This extension fills a gap in minimal JsRuntime configurations where `deno_web` is not included but encoding APIs are still needed.

## Module: `runtime:encoding`

```typescript
import { TextEncoder, TextDecoder } from "runtime:encoding";
```

## Usage Example

```typescript
import { TextEncoder, TextDecoder } from "runtime:encoding";

// Encode string to UTF-8 bytes
const encoder = new TextEncoder();
const bytes = encoder.encode("Hello, World!");
console.log(bytes); // Uint8Array [72, 101, 108, 108, 111, ...]

// Decode UTF-8 bytes back to string
const decoder = new TextDecoder();
const text = decoder.decode(bytes);
console.log(text); // "Hello, World!"

// Decode with options
const utf8Decoder = new TextDecoder("utf-8", { fatal: true });
try {
  const result = utf8Decoder.decode(invalidBytes);
} catch (e) {
  console.error("Invalid UTF-8 sequence");
}
```

## TextEncoder API

```typescript
class TextEncoder {
  // The encoding type (always "utf-8")
  readonly encoding: string;

  // Encode a string into a Uint8Array
  encode(input?: string): Uint8Array;

  // Encode into an existing Uint8Array buffer
  encodeInto(source: string, destination: Uint8Array): {
    read: number;    // Characters read from source
    written: number; // Bytes written to destination
  };
}
```

## TextDecoder API

```typescript
class TextDecoder {
  constructor(
    label?: string,    // Encoding label (default: "utf-8")
    options?: {
      fatal?: boolean;    // Throw on invalid sequences
      ignoreBOM?: boolean; // Ignore byte-order mark
    }
  );

  // The encoding type
  readonly encoding: string;

  // Whether decoding should fail on errors
  readonly fatal: boolean;

  // Whether BOM is ignored
  readonly ignoreBOM: boolean;

  // Decode bytes to string
  decode(input?: BufferSource, options?: { stream?: boolean }): string;
}
```

## Supported Encodings

The primary encoding is UTF-8, which handles:
- ASCII (0x00-0x7F)
- Extended Latin, Cyrillic, Greek, etc.
- CJK characters
- Emoji and symbols

## File Structure

```text
crates/ext_encoding/
├── src/
│   └── lib.rs        # Extension registration (minimal Rust)
├── ts/
│   └── init.ts       # Pure JavaScript implementation
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Implementation Notes

This extension is implemented entirely in JavaScript within `ts/init.ts`. The Rust code only:
1. Registers the extension with Deno runtime
2. Provides the module shim that loads the JavaScript implementation

This approach is efficient because:
- UTF-8 encoding/decoding is well-optimized in JavaScript engines
- No FFI overhead for simple operations
- Standard Web API compatibility

## Why This Extension Exists

When building a minimal Deno runtime without the full `deno_web` crate, standard Web APIs like `TextEncoder` and `TextDecoder` are not available. This extension provides those APIs without pulling in the entire web platform implementation.

## Related

- [deno_web](https://crates.io/crates/deno_web) - Full Deno web platform (includes encoding)
- [WHATWG Encoding Standard](https://encoding.spec.whatwg.org/) - Official specification
