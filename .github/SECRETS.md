# GitHub Secrets Configuration

This document describes the GitHub secrets required for the `bundle-app.yml` workflow when code signing is enabled.

## Overview

The bundle workflow supports optional code signing for Windows and macOS. These secrets are **only required** if you enable signing via the workflow inputs (`sign_windows: true` or `sign_macos: true`).

The workflow gracefully handles missing secrets and will warn you if signing is requested but credentials are not available.

## Windows Code Signing Secrets

Required when `sign_windows: true`:

| Secret Name | Description | Format |
|------------|-------------|---------|
| `WINDOWS_CERTIFICATE` | Base64-encoded PFX/P12 certificate file | Base64 string |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the PFX certificate | Plain text |

### How to encode your Windows certificate:

```bash
# On macOS/Linux:
base64 -i your-cert.pfx -o cert-base64.txt

# On Windows (PowerShell):
[Convert]::ToBase64String([IO.File]::ReadAllBytes("your-cert.pfx")) | Out-File cert-base64.txt
```

Then copy the contents of `cert-base64.txt` into the `WINDOWS_CERTIFICATE` secret.

## macOS Code Signing Secrets

Required when `sign_macos: true`:

| Secret Name | Description | Format |
|------------|-------------|---------|
| `MACOS_CERTIFICATE` | Base64-encoded P12 certificate file | Base64 string |
| `MACOS_CERTIFICATE_PASSWORD` | Password for the P12 certificate | Plain text |
| `MACOS_SIGNING_IDENTITY` | Code signing identity name | String (e.g., "Developer ID Application: Your Name (TEAMID)") |
| `APPLE_ID` | Apple ID email for notarization | Email address |
| `APPLE_TEAM_ID` | Apple Developer Team ID | 10-character team ID |
| `APPLE_APP_PASSWORD` | App-specific password for notarization | Password string |

### How to encode your macOS certificate:

```bash
# Export from Keychain Access as .p12, then encode:
base64 -i your-cert.p12 -o cert-base64.txt
```

### How to generate an app-specific password:

1. Go to [appleid.apple.com](https://appleid.apple.com)
2. Sign in with your Apple ID
3. Navigate to "App-Specific Passwords"
4. Generate a new password for "Forge Notarization"

### How to find your Team ID:

1. Go to [developer.apple.com/account](https://developer.apple.com/account)
2. Look for "Team ID" in your membership details

## Setting Secrets in GitHub

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add each secret name and value as described above

## Workflow Behavior

- **Without secrets**: The workflow will bundle unsigned packages successfully
- **With `sign_windows: true` but no secrets**: Workflow warns and skips Windows signing
- **With `sign_macos: true` but no secrets**: Workflow warns and skips macOS signing
- **With secrets configured**: Full signing and notarization enabled

## Security Notes

- Never commit certificates or passwords to your repository
- Use repository secrets (not environment secrets) for sensitive credentials
- Rotate app-specific passwords periodically
- Keep certificate passwords strong and unique
- Consider using GitHub Environments for additional protection in production
