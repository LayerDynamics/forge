---
title: "developer-toolkit"
description: Full-featured developer tools with code signing, crypto, and shell integration
slug: examples/developer-toolkit
---

A comprehensive developer toolkit demonstrating advanced Forge extension capabilities.

## Overview

This example shows:
- Code signing with `runtime:codesign`
- File hashing with `runtime:crypto`
- Process spawning with `runtime:process`
- System tray and dialogs
- Persistent preferences with `runtime:storage`

## Features

- File hashing (MD5, SHA-1, SHA-256, SHA-512)
- Code signature verification
- Signing with developer certificates
- Ad-hoc signing for testing
- Binary inspection (file type, architectures)
- System tray quick access
- Preference persistence

## Running

```bash
forge dev examples/developer-toolkit
```

## Capabilities

```toml
[capabilities.fs]
read = ["**/*"]
write = ["**/*"]

[capabilities.process]
allow = ["codesign", "signtool", "file", "lipo"]

[capabilities.codesign]
allowed = true

[capabilities.crypto]
allowed = true
```

## Key Patterns

### File Hashing

```typescript
import { hash } from "runtime:crypto";
import { readFile } from "runtime:fs";

const content = await readFile(path);

const [md5, sha1, sha256, sha512] = await Promise.all([
  hash("md5", content),
  hash("sha1", content),
  hash("sha256", content),
  hash("sha512", content),
]);
```

### Code Signing (macOS)

```typescript
import {
  sign,
  signAdhoc,
  verify,
  listIdentities,
  checkCapabilities
} from "runtime:codesign";

// Check platform capabilities
const caps = checkCapabilities();
// { codesign: true, security: true, platform: "macos" }

// List available signing identities
const identities = await listIdentities();

// Sign with a specific identity
await sign({
  path: "/path/to/app.app",
  identity: "Developer ID Application: ...",
  hardenedRuntime: true,
  deep: true,
  timestampUrl: "http://timestamp.digicert.com"
});

// Or use ad-hoc signing for local testing
await signAdhoc("/path/to/app.app");
```

### Signature Verification

```typescript
const result = await verify("/path/to/signed.app");
// { valid: true, signer: "Developer ID...", timestamp: "..." }

// Get entitlements (macOS)
const entitlements = await getEntitlements("/path/to/app.app");
```

### Binary Inspection

```typescript
import { spawn } from "runtime:process";

// Get file type
const proc = await spawn("file", {
  args: ["-b", path],
  stdout: "piped"
});

let fileType = "";
for await (const line of proc.stdout) {
  fileType += line;
}
await proc.wait();

// Get architectures (macOS)
const lipoProc = await spawn("lipo", {
  args: ["-archs", path],
  stdout: "piped"
});
// Returns: "x86_64 arm64" for universal binaries
```

### System Notifications

```typescript
import { notify } from "runtime:sys";

await notify({
  title: "Signing Complete",
  body: `Successfully signed ${filename}`
});
```

## Platform Support

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| File Hashing | Yes | Yes | Yes |
| Code Signing | codesign | signtool | - |
| Verification | codesign -v | signtool verify | - |
| Identities | security | certutil | - |

## Architecture

```text
Developer Toolkit
       |
       +-- File Hashing (runtime:crypto)
       |      |-- MD5, SHA-1, SHA-256, SHA-512
       |
       +-- Code Signing (runtime:codesign)
       |      |-- List identities
       |      |-- Sign with certificate
       |      |-- Ad-hoc signing
       |      |-- Verify signatures
       |
       +-- Binary Inspection (runtime:process)
       |      |-- file command
       |      |-- lipo (macOS)
       |
       +-- UI (runtime:window)
              |-- Main window
              |-- System tray
              |-- File dialogs
```

## Extending

Add code signing automation:

```typescript
// Batch sign all binaries in a directory
import { readDir } from "runtime:fs";

for await (const entry of readDir("./dist")) {
  if (entry.isFile && entry.name.endsWith(".app")) {
    await sign({
      path: `./dist/${entry.name}`,
      identity: selectedIdentity,
      deep: true
    });
  }
}
```
