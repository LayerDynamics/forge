---
title: "ext_crypto"
description: Cryptographic operations extension providing the runtime:crypto module.
slug: crates/ext-crypto
---

The `ext_crypto` crate provides cryptographic operations for Forge applications through the `runtime:crypto` module.

## Overview

ext_crypto handles:

- **Random generation** - Cryptographically secure random bytes
- **Hashing** - SHA-256, SHA-384, SHA-512
- **HMAC** - Hash-based message authentication codes
- **Encryption** - AES-256-GCM symmetric encryption
- **Key derivation** - PBKDF2 key derivation

## Module: `runtime:crypto`

```typescript
import {
  randomBytes,
  hash,
  hmac,
  encrypt,
  decrypt,
  deriveKey
} from "runtime:crypto";
```

## Key Types

### Error Types

```rust
enum CryptoErrorCode {
    Generic = 8000,
    InvalidAlgorithm = 8001,
    InvalidKeyLength = 8002,
    EncryptionFailed = 8003,
    DecryptionFailed = 8004,
    HashFailed = 8005,
    HmacFailed = 8006,
    KeyGenerationFailed = 8007,
    KeyDerivationFailed = 8008,
    VerificationFailed = 8009,
}

struct CryptoError {
    code: CryptoErrorCode,
    message: String,
}
```

### Algorithm Types

```rust
enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

enum EncryptionAlgorithm {
    Aes256Gcm,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_crypto_random_bytes` | `randomBytes(length)` | Generate random bytes |
| `op_crypto_hash` | `hash(algorithm, data)` | Hash data |
| `op_crypto_hmac` | `hmac(algorithm, key, data)` | Generate HMAC |
| `op_crypto_encrypt` | `encrypt(key, data, nonce?)` | Encrypt with AES-256-GCM |
| `op_crypto_decrypt` | `decrypt(key, data, nonce)` | Decrypt with AES-256-GCM |
| `op_crypto_derive_key` | `deriveKey(password, salt, iterations)` | PBKDF2 key derivation |
| `op_crypto_verify_hmac` | `verifyHmac(algorithm, key, data, signature)` | Verify HMAC |

## Usage Examples

### Random Generation

```typescript
import { randomBytes } from "runtime:crypto";

// Generate 32 random bytes
const key = await randomBytes(32);
const nonce = await randomBytes(12);
```

### Hashing

```typescript
import { hash } from "runtime:crypto";

const digest = await hash("sha256", new TextEncoder().encode("hello"));
console.log("SHA-256:", new Uint8Array(digest));
```

### HMAC

```typescript
import { hmac, verifyHmac } from "runtime:crypto";

const key = new Uint8Array(32);
const data = new TextEncoder().encode("message");

const signature = await hmac("sha256", key, data);
const valid = await verifyHmac("sha256", key, data, signature);
```

### Encryption/Decryption

```typescript
import { encrypt, decrypt, randomBytes } from "runtime:crypto";

const key = await randomBytes(32);   // 256-bit key
const plaintext = new TextEncoder().encode("secret message");

// Encrypt
const { ciphertext, nonce } = await encrypt(key, plaintext);

// Decrypt
const decrypted = await decrypt(key, ciphertext, nonce);
```

### Key Derivation

```typescript
import { deriveKey } from "runtime:crypto";

const password = new TextEncoder().encode("password");
const salt = await randomBytes(16);

// Derive 256-bit key using PBKDF2
const key = await deriveKey(password, salt, 100000);
```

## File Structure

```text
crates/ext_crypto/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_enum};
use serde::{Deserialize, Serialize};

#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

#[weld_op(async)]
#[op2(async)]
#[buffer]
pub async fn op_crypto_hash(
    #[serde] algorithm: HashAlgorithm,
    #[buffer] data: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_crypto", "runtime:crypto")
        .ts_path("ts/init.ts")
        .ops(&["op_crypto_random_bytes", "op_crypto_hash", "op_crypto_encrypt", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_crypto extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `ring` | Cryptographic primitives |
| `serde` | Serialization |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Security Notes

- Uses the `ring` library for cryptographic operations
- AES-256-GCM provides authenticated encryption
- PBKDF2 uses HMAC-SHA256 internally
- Random bytes are from system's secure random source

## Related

- [Architecture](/docs/architecture) - Full system architecture
