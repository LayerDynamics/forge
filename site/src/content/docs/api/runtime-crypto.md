---
title: "runtime:crypto"
description: Cryptographic operations including random generation, hashing, HMAC, and symmetric encryption.
slug: api/runtime-crypto
---

The `runtime:crypto` module provides cryptographic operations for Forge applications, powered by the [ring](https://github.com/briansmith/ring) cryptography library.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_crypto](/docs/crates/ext-crypto) for implementation details.

## Features

**Random Generation**:
- Cryptographically secure random bytes
- UUID v4 generation

**Hashing**:
- SHA-256, SHA-384, SHA-512
- Binary and hex output formats

**Message Authentication**:
- HMAC with SHA-256/384/512
- Signature creation and verification

**Symmetric Encryption**:
- AES-256-GCM and AES-128-GCM
- Authenticated encryption with associated data (AEAD)
- Automatic IV generation

**Key Management**:
- Random key generation
- PBKDF2 password-based key derivation

---

## Random Generation

### randomBytes(size)

Generate cryptographically secure random bytes.

Uses the operating system's secure random number generator (e.g., `/dev/urandom` on Unix, `BCryptGenRandom` on Windows).

```typescript
import { randomBytes } from "runtime:crypto";

// Generate 32 random bytes
const bytes = randomBytes(32);
console.log(bytes); // => Uint8Array(32) [...]

// Generate random salt for password hashing
const salt = randomBytes(16);

// Generate random encryption key
const key = randomBytes(32); // 256 bits
```

**Parameters:**
- `size` - Number of bytes to generate

**Returns:** `Uint8Array` with cryptographically secure random bytes

**Throws:**
- Error [8007] if random generation fails

### randomUUID()

Generate a random UUID v4.

Returns a standard format UUID string (e.g., `"550e8400-e29b-41d4-a716-446655440000"`).

```typescript
import { randomUUID } from "runtime:crypto";

// Generate unique ID
const id = randomUUID();
console.log(id); // => "550e8400-e29b-41d4-a716-446655440000"

// Use for unique file names
const filename = `${randomUUID()}.tmp`;

// Use for session IDs
const sessionId = randomUUID();
await set("session.current", sessionId);
```

**Returns:** UUID v4 string in standard format

**Throws:**
- Error [8007] if UUID generation fails

---

## Hashing

### hash(algorithm, data)

Hash data using the specified algorithm.

Returns the hash as a `Uint8Array`. For hexadecimal string output, use `hashHex()`.

```typescript
import { hash } from "runtime:crypto";

// Hash string
const text = "Hello, World!";
const textHash = hash("sha256", text);
console.log(textHash); // => Uint8Array(32) [...]

// Hash binary data
const data = new Uint8Array([1, 2, 3, 4]);
const dataHash = hash("sha256", data);

// Hash file contents
const fileBytes = await readBytes("./document.pdf");
const fileHash = hash("sha256", fileBytes);
```

**Supported algorithms:**
- `"sha256"` - SHA-256 (32 bytes / 256 bits)
- `"sha384"` - SHA-384 (48 bytes / 384 bits)
- `"sha512"` - SHA-512 (64 bytes / 512 bits)

**Parameters:**
- `algorithm` - Hash algorithm
- `data` - Data to hash (string or `Uint8Array`)

**Returns:** `Uint8Array` containing the hash

**Throws:**
- Error [8001] if algorithm is invalid
- Error [8005] if hashing fails

### hashHex(algorithm, data)

Hash data and return the result as a hexadecimal string.

Convenience function that combines `hash()` with hex encoding.

```typescript
import { hashHex } from "runtime:crypto";

// Hash and get hex string
const text = "Hello, World!";
const hex = hashHex("sha256", text);
console.log(hex); // => "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"

// Hash password (Note: Use deriveKey for real password hashing!)
const password = "my-password";
const passwordHash = hashHex("sha256", password);

// Verify file integrity
const expected = "dffd6021bb...";
const actual = hashHex("sha256", await readBytes("file.zip"));
if (actual === expected) {
  console.log("File integrity verified!");
}
```

**Parameters:**
- `algorithm` - Hash algorithm (sha256, sha384, sha512)
- `data` - Data to hash (string or `Uint8Array`)

**Returns:** Hexadecimal string representation of the hash

**Throws:**
- Error [8001] if algorithm is invalid
- Error [8005] if hashing fails

---

## Message Authentication

### hmac(algorithm, key, data)

Compute HMAC (Hash-based Message Authentication Code) signature.

HMAC provides both integrity and authenticity verification using a secret key. The recipient must have the same key to verify the signature.

```typescript
import { hmac, randomBytes } from "runtime:crypto";

// Generate secret key
const secretKey = randomBytes(32);

// Create HMAC signature
const message = new TextEncoder().encode("Important message");
const signature = hmac("sha256", secretKey, message);

// Send message + signature
// Recipient can verify with same key
```

**Use for API request signing:**

```typescript
import { hmac } from "runtime:crypto";

function signRequest(method: string, url: string, body: string, apiSecret: Uint8Array) {
  const payload = `${method}\n${url}\n${body}`;
  const payloadBytes = new TextEncoder().encode(payload);
  const signature = hmac("sha256", apiSecret, payloadBytes);

  return {
    signature: Array.from(signature).map(b => b.toString(16).padStart(2, '0')).join(''),
    payload
  };
}
```

**Parameters:**
- `algorithm` - HMAC algorithm (sha256, sha384, sha512)
- `key` - Secret key as `Uint8Array`
- `data` - Data to sign as `Uint8Array`

**Returns:** `Uint8Array` containing the HMAC signature

**Throws:**
- Error [8001] if algorithm is invalid
- Error [8006] if HMAC operation fails

### verify(algorithm, key, data, signature)

Verify an HMAC signature.

Performs constant-time comparison to prevent timing attacks.

```typescript
import { hmac, verify } from "runtime:crypto";

const secretKey = randomBytes(32);
const message = new TextEncoder().encode("Important message");

// Create signature
const signature = hmac("sha256", secretKey, message);

// Verify signature
const isValid = verify("sha256", secretKey, message, signature);
console.log(isValid); // => true

// Verify with wrong key
const wrongKey = randomBytes(32);
const isInvalid = verify("sha256", wrongKey, message, signature);
console.log(isInvalid); // => false
```

**Use for API request verification:**

```typescript
import { verify } from "runtime:crypto";

function verifyRequest(
  method: string,
  url: string,
  body: string,
  receivedSignature: Uint8Array,
  apiSecret: Uint8Array
): boolean {
  const payload = `${method}\n${url}\n${body}`;
  const payloadBytes = new TextEncoder().encode(payload);

  return verify("sha256", apiSecret, payloadBytes, receivedSignature);
}
```

**Parameters:**
- `algorithm` - HMAC algorithm (sha256, sha384, sha512)
- `key` - Secret key used to create the signature
- `data` - Original data that was signed
- `signature` - Signature to verify

**Returns:** `true` if signature is valid, `false` otherwise

**Throws:**
- Error [8001] if algorithm is invalid
- Error [8009] if verification operation fails

---

## Symmetric Encryption

### encrypt(algorithm, key, data, iv?)

Encrypt data using AES-GCM authenticated encryption.

AES-GCM provides both confidentiality (encryption) and authenticity (authentication tag). The authentication tag ensures the ciphertext hasn't been tampered with.

```typescript
import { encrypt, generateKey } from "runtime:crypto";

// Generate encryption key
const key = generateKey("aes-256-gcm");

// Encrypt data (IV generated automatically)
const plaintext = new TextEncoder().encode("Secret message");
const encrypted = encrypt("aes-256-gcm", key, plaintext);

console.log(encrypted.ciphertext); // Encrypted data
console.log(encrypted.iv);         // Initialization vector (12 bytes)
console.log(encrypted.tag);        // Authentication tag (16 bytes)

// Store all three components to decrypt later
await set("encrypted.data", {
  ciphertext: Array.from(encrypted.ciphertext),
  iv: Array.from(encrypted.iv),
  tag: Array.from(encrypted.tag)
});
```

**Custom IV (advanced):**

```typescript
import { encrypt, randomBytes } from "runtime:crypto";

// Provide your own IV (must be 12 bytes for AES-GCM)
const key = generateKey("aes-256-gcm");
const iv = randomBytes(12);
const plaintext = new TextEncoder().encode("Secret");

const encrypted = encrypt("aes-256-gcm", key, plaintext, iv);
```

**Supported algorithms:**
- `"aes-256-gcm"` - AES-256-GCM (32-byte key, 12-byte IV)
- `"aes-128-gcm"` - AES-128-GCM (16-byte key, 12-byte IV)

**Parameters:**
- `algorithm` - Encryption algorithm
- `key` - Encryption key (32 bytes for AES-256, 16 bytes for AES-128)
- `data` - Data to encrypt as `Uint8Array`
- `iv` - Optional 12-byte initialization vector (generated if not provided)

**Returns:** `EncryptResult` object:
- `ciphertext` - Encrypted data
- `iv` - Initialization vector (12 bytes)
- `tag` - Authentication tag (16 bytes)

**Throws:**
- Error [8001] if algorithm is invalid
- Error [8002] if key length is invalid
- Error [8003] if encryption fails

### decrypt(algorithm, key, encrypted)

Decrypt data encrypted with AES-GCM.

Verifies the authentication tag before decrypting. If the tag is invalid (data was tampered with), decryption fails.

```typescript
import { decrypt, encrypt, generateKey } from "runtime:crypto";

// Encrypt
const key = generateKey("aes-256-gcm");
const plaintext = new TextEncoder().encode("Secret message");
const encrypted = encrypt("aes-256-gcm", key, plaintext);

// Decrypt
const decrypted = decrypt("aes-256-gcm", key, encrypted);
const message = new TextDecoder().decode(decrypted);
console.log(message); // => "Secret message"
```

**Load and decrypt from storage:**

```typescript
import { decrypt } from "runtime:crypto";

const storedData = await get("encrypted.data");
const encrypted = {
  ciphertext: new Uint8Array(storedData.ciphertext),
  iv: new Uint8Array(storedData.iv),
  tag: new Uint8Array(storedData.tag)
};

const key = loadEncryptionKey(); // Load key securely
const decrypted = decrypt("aes-256-gcm", key, encrypted);
const message = new TextDecoder().decode(decrypted);
```

**Parameters:**
- `algorithm` - Decryption algorithm (must match encryption algorithm)
- `key` - Decryption key (must match encryption key)
- `encrypted` - `EncryptResult` object with ciphertext, IV, and tag

**Returns:** `Uint8Array` containing decrypted plaintext

**Throws:**
- Error [8001] if algorithm is invalid
- Error [8002] if key length is invalid
- Error [8004] if decryption fails (wrong key or tampered data)

---

## Key Management

### generateKey(algorithm, length?)

Generate a random encryption or HMAC key.

Uses cryptographically secure random number generation.

```typescript
import { generateKey } from "runtime:crypto";

// Generate AES-256 key (32 bytes)
const aes256Key = generateKey("aes-256-gcm");
console.log(aes256Key.length); // => 32

// Generate AES-128 key (16 bytes)
const aes128Key = generateKey("aes-128-gcm");
console.log(aes128Key.length); // => 16

// Generate HMAC key (default: 32 bytes)
const hmacKey = generateKey("hmac-sha256");

// Generate HMAC key with custom length
const longHmacKey = generateKey("hmac-sha256", 64);
console.log(longHmacKey.length); // => 64
```

**Store key securely:**

```typescript
import { generateKey } from "runtime:crypto";

// Generate key
const key = generateKey("aes-256-gcm");

// Store in secure location (NOT in storage!)
// Consider using OS keychain/credential manager
await writeBytes(
  `${getPath("appData")}/.keys/encryption.key`,
  key
);
```

**Supported algorithms:**
- `"aes-256-gcm"` - Returns 32-byte key
- `"aes-128-gcm"` - Returns 16-byte key
- `"hmac-sha256"` - Returns 32-byte key (or custom length)
- `"hmac-sha384"` - Returns 48-byte key (or custom length)
- `"hmac-sha512"` - Returns 64-byte key (or custom length)

**Parameters:**
- `algorithm` - Algorithm for the key
- `length` - Optional key length in bytes (for HMAC keys only)

**Returns:** `Uint8Array` containing the generated key

**Throws:**
- Error [8001] if algorithm is invalid
- Error [8007] if key generation fails

### deriveKey(password, salt, iterations, keyLength)

Derive a key from a password using PBKDF2.

PBKDF2 (Password-Based Key Derivation Function 2) converts a password into a cryptographic key through many iterations, making brute-force attacks computationally expensive.

```typescript
import { deriveKey, randomBytes } from "runtime:crypto";

// Derive encryption key from password
const password = "user-password-123";
const salt = randomBytes(16); // Generate random salt
const iterations = 100_000;   // High iteration count for security
const keyLength = 32;          // 256 bits for AES-256

const key = deriveKey(password, salt, iterations, keyLength);

// Store salt (NOT the key!) - needed for future derivations
await set("user.salt", Array.from(salt));

// Use key for encryption
const plaintext = new TextEncoder().encode("Secret data");
const encrypted = encrypt("aes-256-gcm", key, plaintext);
```

**Verify password by deriving key again:**

```typescript
import { deriveKey } from "runtime:crypto";

async function verifyPassword(inputPassword: string): Promise<boolean> {
  // Load stored salt
  const storedSalt = await get<number[]>("user.salt");
  const salt = new Uint8Array(storedSalt);

  // Derive key from input password
  const derivedKey = deriveKey(inputPassword, salt, 100_000, 32);

  // Try to decrypt known encrypted data
  try {
    const encryptedData = await get<EncryptedData>("test.data");
    decrypt("aes-256-gcm", derivedKey, encryptedData);
    return true; // Decryption succeeded = correct password
  } catch {
    return false; // Decryption failed = wrong password
  }
}
```

**Recommended parameters:**
- **Salt**: At least 16 bytes (128 bits), randomly generated
- **Iterations**: 100,000+ (higher is more secure but slower)
- **Key length**: 32 bytes for AES-256, 16 bytes for AES-128

**Parameters:**
- `password` - Password string
- `salt` - Salt bytes (at least 8 bytes, 16+ recommended)
- `iterations` - Number of iterations (10,000+ recommended, 100,000+ preferred)
- `keyLength` - Desired key length in bytes

**Returns:** `Uint8Array` containing the derived key

**Throws:**
- Error [8008] if key derivation fails

---

## Type Definitions

```typescript
type HashAlgorithm = "sha256" | "sha384" | "sha512";

type EncryptionAlgorithm = "aes-256-gcm" | "aes-128-gcm";

interface EncryptResult {
  ciphertext: Uint8Array;  // Encrypted data
  iv: Uint8Array;          // Initialization vector (12 bytes)
  tag: Uint8Array;         // Authentication tag (16 bytes)
}
```

---

## Lifecycle Hooks

Intercept crypto operations with before/after/error hooks:

### onBefore(opName, handler)

Execute before an operation:

```typescript
import { onBefore } from "runtime:crypto";

onBefore("encrypt", (args) => {
  console.log("Encrypting data...");
});
```

### onAfter(opName, handler)

Execute after successful operation:

```typescript
import { onAfter } from "runtime:crypto";

onAfter("encrypt", (result, args) => {
  console.log("Encryption successful");
});
```

### onError(opName, handler)

Execute when operation fails:

```typescript
import { onError } from "runtime:crypto";

onError("decrypt", (error, args) => {
  console.error("Decryption failed:", error);
});
```

### removeAllHooks(opName?)

Remove all hooks for an operation (or all operations if no name provided):

```typescript
import { removeAllHooks } from "runtime:crypto";

// Remove all hooks for specific operation
removeAllHooks("encrypt");

// Remove all hooks for all operations
removeAllHooks();
```

**Supported operations:**
`randomBytes`, `randomUuid`, `hash`, `hashHex`, `hmac`, `encrypt`, `decrypt`, `generateKey`, `deriveKey`, `verify`

---

## Handler System

Register custom named handlers for crypto operations:

### registerHandler(name, handler)

Register a named handler:

```typescript
import { registerHandler } from "runtime:crypto";

registerHandler("hashPassword", async (password: string) => {
  const salt = randomBytes(16);
  const key = deriveKey(password, salt, 100_000, 32);
  return { key, salt };
});
```

### invokeHandler(name, ...args)

Invoke a handler by name:

```typescript
import { invokeHandler } from "runtime:crypto";

const { key, salt } = await invokeHandler("hashPassword", "user-password");
```

### listHandlers()

List all registered handlers:

```typescript
import { listHandlers } from "runtime:crypto";

const handlers = listHandlers();
console.log("Registered handlers:", handlers);
```

### hasHandler(name)

Check if a handler exists:

```typescript
import { hasHandler } from "runtime:crypto";

if (hasHandler("hashPassword")) {
  const result = await invokeHandler("hashPassword", "password");
}
```

### removeHandler(name)

Unregister a handler:

```typescript
import { removeHandler } from "runtime:crypto";

removeHandler("hashPassword");
```

---

## Error Handling

All operations throw on error:

```typescript
import { decrypt } from "runtime:crypto";

try {
  const decrypted = decrypt("aes-256-gcm", key, encrypted);
} catch (error) {
  if (error.message.includes("8004")) {
    console.log("Decryption failed - wrong key or tampered data");
  } else if (error.message.includes("8002")) {
    console.log("Invalid key length");
  }
}
```

---

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| `8000` | Generic | Unspecified crypto error |
| `8001` | InvalidAlgorithm | Invalid algorithm specified |
| `8002` | InvalidKeyLength | Invalid key length for algorithm |
| `8003` | EncryptionFailed | Encryption operation failed |
| `8004` | DecryptionFailed | Decryption failed (wrong key or tampered data) |
| `8005` | HashFailed | Hash operation failed |
| `8006` | HmacFailed | HMAC operation failed |
| `8007` | KeyGenerationFailed | Key generation failed |
| `8008` | KeyDerivationFailed | Key derivation (PBKDF2) failed |
| `8009` | VerificationFailed | Signature verification failed |

---

## Security Best Practices

### Key Storage

**Never store encryption keys in plain text!**

```typescript
// ❌ BAD: Storing key in storage
const key = generateKey("aes-256-gcm");
await set("encryption.key", Array.from(key)); // INSECURE!

// ✅ GOOD: Derive key from user password
const password = await promptPassword();
const salt = randomBytes(16);
const key = deriveKey(password, salt, 100_000, 32);
await set("user.salt", Array.from(salt)); // Salt is public, key is not stored
```

### Password Hashing

**Use deriveKey() with high iterations, not hash():**

```typescript
// ❌ BAD: Simple hashing
const passwordHash = hashHex("sha256", password); // Vulnerable to rainbow tables!

// ✅ GOOD: PBKDF2 with salt
const salt = randomBytes(16);
const derivedKey = deriveKey(password, salt, 100_000, 32);
```

### Encryption

**Always use authenticated encryption (AES-GCM):**

```typescript
// ✅ GOOD: AES-GCM provides authentication
const encrypted = encrypt("aes-256-gcm", key, plaintext);
// The 'tag' ensures data hasn't been tampered with
```

### Random Generation

**Use cryptographic random for security-sensitive operations:**

```typescript
// ✅ GOOD: Cryptographic random
const token = randomBytes(32);
const sessionId = randomUUID();

// ❌ BAD: Math.random() is NOT cryptographically secure!
const insecureToken = Math.random(); // DO NOT USE FOR SECURITY
```

---

## Complete Example

```typescript
import {
  generateKey,
  encrypt,
  decrypt,
  hash,
  hashHex,
  hmac,
  verify,
  deriveKey,
  randomBytes,
  randomUUID
} from "runtime:crypto";
import { set, get } from "runtime:storage";

// Example: Secure file encryption with password

async function encryptFile(filePath: string, password: string) {
  // Read file
  const fileData = await readBytes(filePath);

  // Generate salt and derive key from password
  const salt = randomBytes(16);
  const key = deriveKey(password, salt, 100_000, 32);

  // Encrypt file data
  const encrypted = encrypt("aes-256-gcm", key, fileData);

  // Store encrypted data and salt (NOT the key!)
  await set("encrypted.file", {
    ciphertext: Array.from(encrypted.ciphertext),
    iv: Array.from(encrypted.iv),
    tag: Array.from(encrypted.tag),
    salt: Array.from(salt)
  });

  console.log("File encrypted successfully");
}

async function decryptFile(password: string): Promise<Uint8Array> {
  // Load encrypted data
  const stored = await get<any>("encrypted.file");

  // Reconstruct salt and encrypted data
  const salt = new Uint8Array(stored.salt);
  const encrypted = {
    ciphertext: new Uint8Array(stored.ciphertext),
    iv: new Uint8Array(stored.iv),
    tag: new Uint8Array(stored.tag)
  };

  // Derive key from password
  const key = deriveKey(password, salt, 100_000, 32);

  // Decrypt
  try {
    const decrypted = decrypt("aes-256-gcm", key, encrypted);
    console.log("File decrypted successfully");
    return decrypted;
  } catch (error) {
    throw new Error("Decryption failed - wrong password or corrupted data");
  }
}

// Example: API request signing with HMAC

function signApiRequest(
  method: string,
  url: string,
  body: string,
  apiSecret: Uint8Array
): { signature: string; timestamp: string } {
  const timestamp = new Date().toISOString();
  const payload = `${method}\n${url}\n${timestamp}\n${body}`;
  const payloadBytes = new TextEncoder().encode(payload);

  // Create HMAC signature
  const signature = hmac("sha256", apiSecret, payloadBytes);
  const signatureHex = Array.from(signature)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');

  return { signature: signatureHex, timestamp };
}

function verifyApiRequest(
  method: string,
  url: string,
  body: string,
  timestamp: string,
  receivedSignature: string,
  apiSecret: Uint8Array
): boolean {
  const payload = `${method}\n${url}\n${timestamp}\n${body}`;
  const payloadBytes = new TextEncoder().encode(payload);

  // Convert hex signature to bytes
  const signatureBytes = new Uint8Array(
    receivedSignature.match(/.{2}/g)!.map(h => parseInt(h, 16))
  );

  // Verify signature
  return verify("sha256", apiSecret, payloadBytes, signatureBytes);
}

// Example: Secure session token generation

async function createSession(userId: string): Promise<string> {
  // Generate secure session token
  const sessionToken = randomUUID();
  const tokenBytes = new TextEncoder().encode(sessionToken);

  // Create HMAC to prevent tampering
  const secretKey = loadServerSecret(); // Load from secure storage
  const signature = hmac("sha256", secretKey, tokenBytes);

  // Store session with signature
  await set(`session.${sessionToken}`, {
    userId,
    signature: Array.from(signature),
    createdAt: Date.now()
  });

  return sessionToken;
}

async function verifySession(sessionToken: string): Promise<string | null> {
  const session = await get<any>(`session.${sessionToken}`);
  if (!session) return null;

  // Verify session hasn't been tampered with
  const tokenBytes = new TextEncoder().encode(sessionToken);
  const secretKey = loadServerSecret();
  const storedSignature = new Uint8Array(session.signature);

  const isValid = verify("sha256", secretKey, tokenBytes, storedSignature);

  return isValid ? session.userId : null;
}
```
