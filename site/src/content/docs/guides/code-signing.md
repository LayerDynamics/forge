---
title: Code Signing Guide
description: Sign your Forge app for distribution on macOS, Windows, and Linux.
slug: guides/code-signing
---

Code signing verifies your app's authenticity and integrity. It's required for distribution on macOS (especially for notarization) and recommended on Windows for user trust.

## Overview

Forge provides code signing through:

1. **Automatic signing** during `forge bundle` (via manifest configuration)
2. **Manual signing** with `forge sign` command
3. **Environment variables** for CI/CD integration

---

## macOS Code Signing

### Prerequisites

- **Apple Developer Program** membership ($99/year)
- **Xcode Command Line Tools**: `xcode-select --install`
- **Developer ID Application** certificate in Keychain

### Setting Up Your Signing Identity

1. Create a Developer ID certificate in [Apple Developer Portal](https://developer.apple.com/account/resources/certificates)
2. Download and install in Keychain Access
3. Verify installation:

```bash
security find-identity -v -p codesigning
```

You should see output like:

```
1) ABC123DEF "Developer ID Application: Your Name (TEAMID)"
```

### Manifest Configuration

Configure signing in your `manifest.app.toml`:

```toml
[bundle.macos]
sign = true
signing_identity = "Developer ID Application: Your Name (TEAMID)"
team_id = "TEAMID"

# Optional: custom entitlements
entitlements = "entitlements.plist"

# Optional: notarization (requires signing)
notarize = true

# App Store category
category = "public.app-category.developer-tools"

# Minimum macOS version
minimum_system_version = "12.0"
```

### Notarization

Apple notarization is required for apps distributed outside the App Store on macOS 10.15+.

**One-time setup** - store your credentials:

```bash
xcrun notarytool store-credentials forge-notarize
```

You'll be prompted for:
- Apple ID email
- Team ID (from Apple Developer Portal)
- App-specific password (generate at [appleid.apple.com](https://appleid.apple.com))

**Enable notarization** in manifest:

```toml
[bundle.macos]
sign = true
notarize = true
team_id = "TEAMID"
signing_identity = "Developer ID Application: Your Name (TEAMID)"
```

### Build and Bundle

```bash
# Build and bundle with automatic signing + notarization
forge build .
forge bundle .
```

Output:

```
Creating macOS app bundle...
  Building release binary with embedded assets...
  Generating Info.plist...
  Generating icon...
  Signing app bundle...
  Creating DMG...
  Submitting for notarization...
    Notarization successful
    Stapled notarization ticket

  App bundle: bundle/MyApp.app
  DMG: bundle/MyApp-1.0.0-macos.dmg
```

### Manual Signing

Sign an existing artifact:

```bash
forge sign ./bundle/MyApp.app --identity "Developer ID Application: Your Name (TEAMID)"
```

With notarization:

```bash
export FORGE_TEAM_ID="TEAMID"
export FORGE_NOTARIZE=1
forge sign ./bundle/MyApp-1.0.0-macos.dmg --identity "Developer ID Application: Your Name (TEAMID)"
```

### Ad-hoc Signing (Development)

For local testing without a certificate:

```bash
forge sign ./bundle/MyApp.app --identity "-"
```

Ad-hoc signed apps will show security warnings to users and cannot be notarized.

---

## Windows Code Signing

### Prerequisites

- **Windows SDK** with SignTool (install via Visual Studio Installer or standalone)
- **Code signing certificate** (.pfx file)
  - Purchase from a Certificate Authority (DigiCert, Sectigo, etc.)
  - Or use a self-signed certificate for testing

### SignTool Location

Forge automatically searches for SignTool in:

1. System PATH
2. `C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64\signtool.exe`
3. `C:\Program Files (x86)\Windows Kits\10\bin\<version>\x86\signtool.exe`

### Creating a Test Certificate

For development/testing only:

```powershell
# Generate self-signed certificate
$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject "CN=My Company" `
  -KeyExportPolicy Exportable `
  -CertStoreLocation Cert:\CurrentUser\My

# Export to .pfx file
$pwd = ConvertTo-SecureString -String "your-password" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath "cert.pfx" -Password $pwd
```

**Warning:** Self-signed certificates will trigger SmartScreen warnings. Use a trusted CA certificate for production.

### Manifest Configuration

```toml
[bundle.windows]
format = "msix"
sign = true
certificate = "cert.pfx"
password = "$CERT_PASSWORD"        # References environment variable
publisher = "CN=My Company, O=My Company, C=US"
min_version = "10.0.17763.0"
capabilities = ["internetClient"]
```

**Password options:**
- `password = "$ENV_VAR"` - Read from environment variable (recommended)
- `password = "literal"` - Literal password (avoid in version control)

### Build and Bundle

```bash
# Set certificate password
export CERT_PASSWORD="your-password"

# Build and bundle
forge build .
forge bundle .
```

### Manual Signing

```bash
export FORGE_SIGNING_PASSWORD="your-password"
forge sign ./bundle/MyApp-1.0.0.msix --identity "cert.pfx"
```

---

## Linux Code Signing

Linux code signing is optional and uses GPG for creating detached signatures.

### Prerequisites

- **GPG** installed and configured
- A GPG key pair

### Creating a GPG Key

```bash
gpg --gen-key
```

### Signing

Linux AppImages are signed automatically if GPG is available:

```bash
forge bundle .
```

Or manually:

```bash
forge sign ./bundle/MyApp-1.0.0.AppImage --identity "your.email@example.com"
```

This creates a detached signature at `MyApp-1.0.0.AppImage.sig`.

### Verification

Users can verify:

```bash
gpg --verify MyApp-1.0.0.AppImage.sig MyApp-1.0.0.AppImage
```

**Note:** GPG signing is optional. If GPG is not available or signing fails, the bundle process continues without error.

---

## Environment Variables

Use environment variables for CI/CD integration:

| Variable | Platform | Description |
|----------|----------|-------------|
| `FORGE_SIGNING_IDENTITY` | All | Signing identity (certificate path or name) |
| `CODESIGN_IDENTITY` | macOS | Fallback for signing identity |
| `FORGE_SIGNING_PASSWORD` | Windows | Certificate password |
| `FORGE_TEAM_ID` | macOS | Apple Developer Team ID |
| `FORGE_NOTARIZE` | macOS | Enable notarization (any value) |

### CI/CD Example (GitHub Actions)

```yaml
jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Import certificate
        env:
          CERTIFICATE_BASE64: ${{ secrets.APPLE_CERTIFICATE }}
          CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        run: |
          echo $CERTIFICATE_BASE64 | base64 --decode > certificate.p12
          security create-keychain -p "" build.keychain
          security import certificate.p12 -k build.keychain -P "$CERTIFICATE_PASSWORD" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple: -s -k "" build.keychain

      - name: Setup notarization credentials
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          APPLE_APP_PASSWORD: ${{ secrets.APPLE_APP_PASSWORD }}
        run: |
          xcrun notarytool store-credentials forge-notarize \
            --apple-id "$APPLE_ID" \
            --team-id "$APPLE_TEAM_ID" \
            --password "$APPLE_APP_PASSWORD"

      - name: Build and bundle
        env:
          FORGE_SIGNING_IDENTITY: "Developer ID Application: Your Name (TEAMID)"
          FORGE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          FORGE_NOTARIZE: "1"
        run: |
          forge build .
          forge bundle .

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Import certificate
        env:
          CERTIFICATE_BASE64: ${{ secrets.WINDOWS_CERTIFICATE }}
        run: |
          $bytes = [Convert]::FromBase64String($env:CERTIFICATE_BASE64)
          [IO.File]::WriteAllBytes("cert.pfx", $bytes)

      - name: Build and bundle
        env:
          FORGE_SIGNING_IDENTITY: "cert.pfx"
          FORGE_SIGNING_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
        run: |
          forge build .
          forge bundle .
```

---

## CLI Reference

### forge sign

Sign a bundled artifact.

```
forge sign [OPTIONS] <ARTIFACT>
```

**Arguments:**
- `<ARTIFACT>` - Path to artifact (`.app`, `.dmg`, `.msix`, `.exe`, `.AppImage`)

**Options:**
- `--identity <IDENTITY>` or `-i` - Signing identity

**Examples:**

```bash
# macOS bundle
forge sign MyApp.app --identity "Developer ID Application: Name (TEAMID)"

# macOS DMG with notarization
FORGE_NOTARIZE=1 FORGE_TEAM_ID=TEAMID forge sign MyApp.dmg --identity "Developer ID..."

# Windows MSIX
FORGE_SIGNING_PASSWORD=secret forge sign MyApp.msix --identity cert.pfx

# Linux AppImage
forge sign MyApp.AppImage --identity user@example.com
```

---

## Troubleshooting

### macOS

**"codesign: command not found"**
- Install Xcode Command Line Tools: `xcode-select --install`

**"No identity found"**
- Verify certificate is in Keychain: `security find-identity -v -p codesigning`
- Ensure the full identity string matches exactly

**"Notarization failed"**
- Verify credentials: `xcrun notarytool store-credentials forge-notarize`
- Check Apple Developer account status
- Ensure hardened runtime is enabled (automatic with Forge)

**"The signature is invalid"**
- Re-sign with `--force` flag (automatic with Forge)
- Check for modified files after signing

### Windows

**"SignTool not found"**
- Install Windows SDK via Visual Studio Installer
- Add SignTool to PATH or let Forge auto-discover

**"Certificate not valid for code signing"**
- Ensure certificate has Code Signing EKU
- Check certificate expiration date

**SmartScreen warnings**
- Use an EV (Extended Validation) certificate for immediate trust
- Standard certificates build reputation over time

### Linux

**"gpg: command not found"**
- Install GPG: `apt install gnupg` or equivalent
- GPG signing is optional; builds continue without it

**"No secret key"**
- Ensure GPG key exists: `gpg --list-secret-keys`
- Generate a key: `gpg --gen-key`
