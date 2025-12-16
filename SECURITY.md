# Security Policy

Forge is a desktop application framework that provides native system access through TypeScript. Security is a core concern given the framework's capability-based permission model and IPC mechanisms.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

As an alpha project, we only support the latest release. Users should always update to the newest version.

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### Private Disclosure (Preferred)

For sensitive security issues, please use **GitHub Security Advisories**:

1. Go to [Security Advisories](https://github.com/LayerDynamics/forge/security/advisories)
2. Click "Report a vulnerability"
3. Provide detailed information about the issue

This allows us to coordinate a fix before public disclosure.

### Alternative Contact

If you cannot use GitHub Security Advisories:

- **Email:** [layerdynamics@proton.me](mailto:layerdynamics@proton.me) (include "SECURITY" in subject)
- **GitHub:** Open a [private security advisory](https://github.com/LayerDynamics/forge/security/advisories/new)

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Initial Response:** Within 48 hours
- **Status Update:** Within 7 days
- **Fix Timeline:** Depends on severity (critical issues prioritized)

### After Reporting

1. We will acknowledge receipt of your report
2. We will investigate and determine the impact
3. We will develop and test a fix
4. We will release a patched version
5. We will publicly disclose the issue (crediting you, unless you prefer anonymity)

## Security Scope

### In Scope

The following are considered security issues:

#### Runtime Security

- Sandbox escapes allowing unauthorized system access
- Capability/permission bypasses in `runtime:*` modules
- IPC vulnerabilities allowing cross-window attacks
- Memory safety issues in Rust runtime code
- Arbitrary code execution vulnerabilities

#### Extension Security

- `ext_fs`: Unauthorized file system access beyond granted permissions
- `ext_ui`: Window spoofing, clickjacking, or IPC message injection
- `ext_net`: Request smuggling or unauthorized network access
- `ext_sys`: Unauthorized system information disclosure
- `ext_process`: Process injection or privilege escalation
- `ext_wasm`: WASM sandbox escapes or memory corruption

#### Build & Distribution

- Supply chain vulnerabilities in the build process
- Code signing bypass in `forge sign`
- Bundle tampering in `forge bundle`

#### WebView Security

- Cross-site scripting (XSS) via `app://` protocol
- Injection attacks through the `window.host` bridge
- Insecure content loading

### Out of Scope

The following are generally not considered security issues:

- Vulnerabilities in user-created applications (not the framework)
- Issues requiring physical access to the machine
- Social engineering attacks
- Denial of service that requires authenticated access
- Security issues in dependencies (report upstream, but notify us)
- Theoretical vulnerabilities without proof of concept

## Security Best Practices for App Developers

When building apps with Forge:

### Capability Permissions

```toml
# manifest.app.toml - Request minimum necessary permissions
[capabilities]
fs = ["read:./data", "write:./data"]  # Scoped, not blanket access
net = ["https://api.example.com"]     # Specific domains only
```

### IPC Security

```typescript
// Validate all IPC messages from renderer
for await (const event of windowEvents()) {
  // Always validate channel and payload
  if (!isValidChannel(event.channel)) continue;
  if (!validatePayload(event.payload)) continue;

  // Process validated message
}
```

### Content Security

```html
<!-- Use restrictive CSP in your web content -->
<meta http-equiv="Content-Security-Policy"
      content="default-src 'self' app:; script-src 'self'">
```

## Security Features

Forge includes several security mechanisms:

1. **Capability-Based Permissions** - Apps must declare required system access in `manifest.app.toml`
2. **IPC Isolation** - Renderer processes communicate through controlled channels
3. **Sandboxed WebViews** - UI runs in system WebView with limited capabilities
4. **No Node.js** - Using Deno eliminates common Node.js security pitfalls

## Acknowledgments

We thank the following researchers for responsibly disclosing security issues:

*No vulnerabilities reported yet.*

---

Thank you for helping keep Forge and its users safe.
