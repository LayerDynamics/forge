//! Windows implementation for custom URL protocol handling
//!
//! Uses Windows Registry for protocol registration.
//! Protocol handlers are registered under HKEY_CLASSES_ROOT\{scheme}
//! with shell\open\command pointing to the executable.

use crate::{
    ProtocolCapabilities, ProtocolError, RegistrationOptions, RegistrationResult,
    RegistrationStatus,
};
use tracing::{debug, error, warn};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// Register a custom URL protocol handler on Windows
///
/// Creates registry entries under HKEY_CLASSES_ROOT:
/// ```text
/// HKEY_CLASSES_ROOT\
///   myapp\
///     (Default) = "URL:MyApp Protocol"
///     URL Protocol = ""
///     DefaultIcon\
///       (Default) = "C:\path\to\app.exe,0"
///     shell\
///       open\
///         command\
///           (Default) = "C:\path\to\app.exe" "%1"
/// ```
#[cfg(target_os = "windows")]
pub async fn register_protocol(
    scheme: &str,
    _app_identifier: &str,
    app_name: &str,
    exe_path: &str,
    options: &RegistrationOptions,
) -> Result<RegistrationResult, ProtocolError> {
    debug!(scheme = %scheme, exe = %exe_path, "Windows: register_protocol");

    // Check if already registered
    let current_status = is_registered(scheme).await?;
    let was_already_registered = current_status.is_registered;
    let previous_handler = current_status.registered_by.clone();

    // Open HKEY_CURRENT_USER\Software\Classes (doesn't require admin)
    // or HKEY_CLASSES_ROOT (requires admin)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes_path = format!(r"Software\Classes\{}", scheme);

    // Create the main key
    let (scheme_key, _) = hkcu.create_subkey(&classes_path).map_err(|e| {
        ProtocolError::registry_error(format!("Failed to create registry key: {}", e))
    })?;

    // Set the description
    let description = options
        .description
        .clone()
        .unwrap_or_else(|| format!("URL:{} Protocol", app_name));
    scheme_key
        .set_value("", &description)
        .map_err(|e| ProtocolError::registry_error(format!("Failed to set description: {}", e)))?;

    // Set URL Protocol marker (empty string indicates this is a URL protocol)
    scheme_key
        .set_value("URL Protocol", &"")
        .map_err(|e| ProtocolError::registry_error(format!("Failed to set URL Protocol: {}", e)))?;

    // Create DefaultIcon subkey
    let (icon_key, _) = scheme_key.create_subkey("DefaultIcon").map_err(|e| {
        ProtocolError::registry_error(format!("Failed to create DefaultIcon key: {}", e))
    })?;

    let icon_value = options
        .icon_path
        .clone()
        .unwrap_or_else(|| format!("{},0", exe_path));
    icon_key
        .set_value("", &icon_value)
        .map_err(|e| ProtocolError::registry_error(format!("Failed to set icon: {}", e)))?;

    // Create shell\open\command subkey
    let (shell_key, _) = scheme_key
        .create_subkey(r"shell\open\command")
        .map_err(|e| {
            ProtocolError::registry_error(format!("Failed to create command key: {}", e))
        })?;

    // Command with %1 placeholder for the URL
    let command = format!(r#""{}" "%1""#, exe_path);
    shell_key
        .set_value("", &command)
        .map_err(|e| ProtocolError::registry_error(format!("Failed to set command: {}", e)))?;

    debug!(scheme = %scheme, "Windows: Protocol registered successfully");

    Ok(RegistrationResult {
        success: true,
        scheme: scheme.to_string(),
        was_already_registered,
        previous_handler,
    })
}

#[cfg(not(target_os = "windows"))]
pub async fn register_protocol(
    _scheme: &str,
    _app_identifier: &str,
    _app_name: &str,
    _exe_path: &str,
    _options: &RegistrationOptions,
) -> Result<RegistrationResult, ProtocolError> {
    Err(ProtocolError::platform_unsupported(
        "Windows protocol registration is only available on Windows",
    ))
}

/// Unregister a custom URL protocol handler on Windows
#[cfg(target_os = "windows")]
pub async fn unregister_protocol(scheme: &str) -> Result<bool, ProtocolError> {
    debug!(scheme = %scheme, "Windows: unregister_protocol");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes_path = format!(r"Software\Classes\{}", scheme);

    // Delete the entire key tree
    match hkcu.delete_subkey_all(&classes_path) {
        Ok(_) => {
            debug!(scheme = %scheme, "Windows: Protocol unregistered successfully");
            Ok(true)
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                warn!(scheme = %scheme, "Windows: Protocol was not registered");
                Ok(false)
            } else {
                Err(ProtocolError::registry_error(format!(
                    "Failed to delete registry key: {}",
                    e
                )))
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub async fn unregister_protocol(_scheme: &str) -> Result<bool, ProtocolError> {
    Err(ProtocolError::platform_unsupported(
        "Windows protocol unregistration is only available on Windows",
    ))
}

/// Check if a scheme is registered on Windows
#[cfg(target_os = "windows")]
pub async fn is_registered(scheme: &str) -> Result<RegistrationStatus, ProtocolError> {
    debug!(scheme = %scheme, "Windows: is_registered");

    // Check HKEY_CURRENT_USER first (user-level registration)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes_path = format!(r"Software\Classes\{}", scheme);

    if let Ok(scheme_key) = hkcu.open_subkey(&classes_path) {
        // Try to get the command to determine the registered app
        let registered_by = scheme_key
            .open_subkey(r"shell\open\command")
            .ok()
            .and_then(|cmd_key| cmd_key.get_value::<String, _>("").ok())
            .and_then(|cmd| {
                // Extract exe path from command like "C:\path\to\app.exe" "%1"
                if cmd.starts_with('"') {
                    cmd.split('"').nth(1).map(|s| s.to_string())
                } else {
                    cmd.split_whitespace().next().map(|s| s.to_string())
                }
            });

        // Check if this is our app
        let current_exe = std::env::current_exe().ok();
        let is_default = match (&registered_by, &current_exe) {
            (Some(reg), Some(curr)) => reg.to_lowercase() == curr.to_string_lossy().to_lowercase(),
            _ => false,
        };

        return Ok(RegistrationStatus {
            is_registered: true,
            is_default,
            registered_by,
        });
    }

    // Check HKEY_CLASSES_ROOT (system-level, read-only for non-admin)
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    if let Ok(scheme_key) = hkcr.open_subkey(scheme) {
        let registered_by = scheme_key
            .open_subkey(r"shell\open\command")
            .ok()
            .and_then(|cmd_key| cmd_key.get_value::<String, _>("").ok())
            .and_then(|cmd| {
                if cmd.starts_with('"') {
                    cmd.split('"').nth(1).map(|s| s.to_string())
                } else {
                    cmd.split_whitespace().next().map(|s| s.to_string())
                }
            });

        return Ok(RegistrationStatus {
            is_registered: true,
            is_default: false, // Not registered by us if in HKCR
            registered_by,
        });
    }

    Ok(RegistrationStatus {
        is_registered: false,
        is_default: false,
        registered_by: None,
    })
}

#[cfg(not(target_os = "windows"))]
pub async fn is_registered(_scheme: &str) -> Result<RegistrationStatus, ProtocolError> {
    Err(ProtocolError::platform_unsupported(
        "Windows protocol query is only available on Windows",
    ))
}

/// Set this app as the default handler for a scheme on Windows
#[cfg(target_os = "windows")]
pub async fn set_as_default(scheme: &str) -> Result<bool, ProtocolError> {
    debug!(scheme = %scheme, "Windows: set_as_default");

    let exe_path = std::env::current_exe()
        .map_err(|e| ProtocolError::generic(format!("Could not get exe path: {}", e)))?
        .to_string_lossy()
        .to_string();

    let app_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "App".to_string());

    let options = RegistrationOptions::default();
    let result = register_protocol(scheme, "", &app_name, &exe_path, &options).await?;

    Ok(result.success)
}

#[cfg(not(target_os = "windows"))]
pub async fn set_as_default(_scheme: &str) -> Result<bool, ProtocolError> {
    Err(ProtocolError::platform_unsupported(
        "Windows set_as_default is only available on Windows",
    ))
}

/// Check platform capabilities on Windows
pub fn check_capabilities() -> ProtocolCapabilities {
    #[cfg(target_os = "windows")]
    {
        // Check if we can write to registry
        let can_write = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(r"Software\Classes\_forge_test_protocol")
            .map(|(key, _)| {
                // Clean up test key
                let _ = key.delete_subkey("");
                true
            })
            .unwrap_or(false);

        ProtocolCapabilities {
            can_register: can_write,
            can_query: true,
            can_deep_link: true,
            platform: "windows".to_string(),
            notes: if can_write {
                Some("Using HKEY_CURRENT_USER registry".to_string())
            } else {
                Some("Registry write access denied".to_string())
            },
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        ProtocolCapabilities {
            can_register: false,
            can_query: false,
            can_deep_link: false,
            platform: "windows".to_string(),
            notes: Some("Not running on Windows".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_capabilities() {
        let caps = check_capabilities();
        assert_eq!(caps.platform, "windows");
    }
}
