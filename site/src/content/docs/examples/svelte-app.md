---
title: "svelte-app"
description: Secure vault application with SvelteKit, encryption, and file storage
slug: examples/svelte-app
---

A comprehensive secure vault application demonstrating advanced Forge capabilities with SvelteKit frontend.

## Overview

This example shows:
- AES-256-GCM encryption with `runtime:crypto`
- Persistent encrypted storage with `runtime:storage`
- File encryption and management
- Session management with auto-lock
- System tray integration

## Features

- Master password protection with PBKDF2 key derivation
- Encrypted notes with category organization
- Secure file vault with binary encryption
- Password generator with customizable options
- Auto-lock after inactivity (5 minutes)
- System tray for quick access
- Import/export encrypted backups

## Running

```bash
forge dev examples/svelte-app
```

## Capabilities

```toml
[capabilities.channels]
allowed = ["*"]

[capabilities.fs]
read = ["**/*"]
write = ["**/*"]

[capabilities.crypto]
allowed = true
```

## Key Patterns

### Key Derivation

```typescript
import { deriveKey, randomBytes } from "runtime:crypto";

const salt = randomBytes(16);
const key = deriveKey(password, salt, 100000, 32); // PBKDF2
```

### Data Encryption

```typescript
import { encrypt, decrypt } from "runtime:crypto";

function encryptData(data: string, key: Uint8Array) {
  const plaintext = new TextEncoder().encode(data);
  const result = encrypt("aes-256-gcm", key, plaintext);
  return {
    ciphertext: toBase64(result.ciphertext),
    iv: toBase64(result.iv),
    tag: toBase64(result.tag)
  };
}

function decryptData(encrypted, key: Uint8Array): string {
  const plaintext = decrypt("aes-256-gcm", key, {
    ciphertext: fromBase64(encrypted.ciphertext),
    iv: fromBase64(encrypted.iv),
    tag: fromBase64(encrypted.tag)
  });
  return new TextDecoder().decode(plaintext);
}
```

### Secure Storage

```typescript
import { get, set, remove } from "runtime:storage";

// Store encrypted vault metadata
await set("vault:meta", encryptedMeta);

// Store individual encrypted notes
await set(`note:${id}`, encryptedContent);
```

### Auto-Lock Timer

```typescript
let lastActivity = Date.now();

setInterval(() => {
  const idle = Date.now() - lastActivity;
  if (idle > LOCK_TIMEOUT_MS) {
    lockVault();
    sendToWindow(windowId, "vault:state", { locked: true });
  }
}, 10000);
```

### Structured Logging

```typescript
import { infoLog, debug, warn, error, setDefaultWindow } from "runtime:log";

setDefaultWindow(windowId); // Forward logs to DevTools
infoLog("Starting Secure Vault", { version: "1.0.0" });
```

## Security Architecture

```text
Password -> PBKDF2 (100k iterations) -> Session Key
                                            |
                                            v
                              +-------------+-------------+
                              |             |             |
                          Vault Meta    Notes Index   File Index
                          (encrypted)   (encrypted)   (encrypted)
                              |             |
                              v             v
                          Verifier    Individual Notes
                          Token       (encrypted per-note)
```

## Extending

Add biometric unlock on supported platforms:

```typescript
// Future: runtime:auth integration
const authenticated = await biometric.authenticate({
  reason: "Unlock your secure vault"
});
```
