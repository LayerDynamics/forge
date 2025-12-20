---
title: "ext_codesign"
description: Code signing extension providing the runtime:codesign module for signing and verifying app bundles.
slug: crates/ext-codesign
---

The `ext_codesign` crate provides cross-platform code signing capabilities for Forge applications through the `runtime:codesign` module.

## Overview

ext_codesign handles:

- **Code Signing** - Sign binaries and app bundles with signing identities
- **Ad-hoc Signing** - Quick signing without certificates (macOS only)
- **Signature Verification** - Verify existing code signatures
- **Identity Management** - List and inspect available signing certificates
- **Entitlements** - Extract entitlements from signed apps (macOS only)
- **Platform Tools** - Uses native tools (codesign on macOS, signtool on Windows)

## Module: `runtime:codesign`

```typescript
import {
  sign,
  signAdhoc,
  verify,
  listIdentities,
  getIdentityInfo,
  getEntitlements,
  checkCapabilities
} from "runtime:codesign";
```

## Key Types

### Error Types

```rust
enum CodesignErrorCode {
    Generic = 8300,
    PathNotFound = 8301,
    IdentityNotFound = 8302,
    SigningFailed = 8303,
    VerificationFailed = 8304,
    PermissionDenied = 8305,
    PlatformUnsupported = 8306,
    InvalidIdentity = 8307,
    EntitlementsFailed = 8308,
    ToolNotFound = 8309,
    InvalidEntitlements = 8310,
    CertificateExpired = 8311,
    CertificateNotTrusted = 8312,
}
```

### Data Types

```rust
struct SignOptions {
    path: String,                    // Path to file/bundle to sign
    identity: String,                // Certificate name or SHA-1 thumbprint
    entitlements: Option<String>,    // Entitlements file path (macOS)
    hardened_runtime: Option<bool>,  // Enable hardened runtime (macOS)
    deep: Option<bool>,              // Deep sign embedded code (macOS)
    timestamp_url: Option<String>,   // Timestamp server URL (Windows)
}

struct SigningIdentity {
    id: String,              // SHA-1 thumbprint
    name: String,            // Human-readable name
    expires: Option<String>, // Expiration date (ISO 8601)
    valid: bool,             // Currently valid
    identity_type: String,   // "developer_id", "distribution", etc.
}

struct VerifyResult {
    valid: bool,               // Signature validity
    signer: Option<String>,    // Signer identity
    timestamp: Option<String>, // Signature timestamp
    message: String,           // Status message
}

struct CodesignCapabilities {
    codesign: bool,   // macOS codesign available
    security: bool,   // macOS security tool available
    signtool: bool,   // Windows SignTool available
    certutil: bool,   // Windows certutil available
    platform: String, // Current platform
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_codesign_sign` | `sign(options)` | Sign a file or app bundle |
| `op_codesign_sign_adhoc` | `signAdhoc(path)` | Ad-hoc sign without identity (macOS) |
| `op_codesign_verify` | `verify(path)` | Verify code signature |
| `op_codesign_get_entitlements` | `getEntitlements(path)` | Extract entitlements (macOS) |
| `op_codesign_list_identities` | `listIdentities()` | List signing certificates |
| `op_codesign_get_identity_info` | `getIdentityInfo(id)` | Get certificate details |
| `op_codesign_check_capabilities` | `checkCapabilities()` | Check available tools |

## Usage Example

```typescript
import { sign, verify, listIdentities, checkCapabilities } from "runtime:codesign";

// Check what's available on this platform
const caps = checkCapabilities();
console.log(`Platform: ${caps.platform}, codesign: ${caps.codesign}`);

// List available signing identities
const identities = await listIdentities();
for (const id of identities) {
  console.log(`${id.name} (${id.identity_type}) - Valid: ${id.valid}`);
}

// Sign an app bundle
await sign({
  path: "/path/to/MyApp.app",
  identity: "Developer ID Application: My Company",
  hardenedRuntime: true,
  deep: true,
});

// Verify the signature
const result = await verify("/path/to/MyApp.app");
if (result.valid) {
  console.log(`Signed by: ${result.signer}`);
}
```

## Platform Support

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Sign with identity | codesign | signtool | GPG (planned) |
| Ad-hoc signing | Yes | No | No |
| Verify signature | Yes | Yes | Planned |
| List identities | Yes | Yes | No |
| Entitlements | Yes | No | No |

## File Structure

```text
crates/ext_codesign/
├── src/
│   ├── lib.rs        # Extension implementation
│   ├── os_mac.rs     # macOS codesign implementation
│   ├── os_windows.rs # Windows signtool implementation
│   └── os_linux.rs   # Linux stub implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Capability Checker

```rust
trait CodesignCapabilityChecker: Send + Sync + 'static {
    fn check_sign(&self) -> Result<(), String>;
    fn check_verify(&self) -> Result<(), String>;
    fn check_list_identities(&self) -> Result<(), String>;
}
```

## Related

- [forge_cli bundler](/docs/crates/forge-cli) - Uses codesign during app bundling
- [runtime:fs](/docs/crates/ext-fs) - File system operations
