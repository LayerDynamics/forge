//! Linux implementation for custom URL protocol handling
//!
//! Uses freedesktop.org standards:
//! - .desktop files in ~/.local/share/applications/
//! - xdg-mime for setting default handlers
//! - MIME type x-scheme-handler/{scheme}

use crate::{
    ProtocolCapabilities, ProtocolError, RegistrationOptions, RegistrationResult,
    RegistrationStatus,
};
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;
use tracing::{debug, error, warn};

/// Get the path to the applications directory
fn get_applications_dir() -> PathBuf {
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(data_home).join("applications")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share/applications")
    } else {
        PathBuf::from("/tmp/applications")
    }
}

/// Generate a .desktop file name for a scheme
fn get_desktop_file_name(scheme: &str, app_name: &str) -> String {
    format!(
        "{}-{}.desktop",
        app_name.to_lowercase().replace(' ', "-"),
        scheme
    )
}

/// Register a custom URL protocol handler on Linux
///
/// Creates a .desktop file and registers it with xdg-mime:
/// ```ini
/// [Desktop Entry]
/// Version=1.0
/// Type=Application
/// Name=My App
/// Exec=/path/to/app %u
/// Icon=myapp
/// MimeType=x-scheme-handler/myapp;
/// Categories=Utility;
/// NoDisplay=true
/// ```
pub async fn register_protocol(
    scheme: &str,
    _app_identifier: &str,
    app_name: &str,
    exe_path: &str,
    options: &RegistrationOptions,
) -> Result<RegistrationResult, ProtocolError> {
    debug!(scheme = %scheme, exe = %exe_path, "Linux: register_protocol");

    // Check current status
    let current_status = is_registered(scheme).await?;
    let was_already_registered = current_status.is_registered;
    let previous_handler = current_status.registered_by.clone();

    // Ensure applications directory exists
    let apps_dir = get_applications_dir();
    fs::create_dir_all(&apps_dir).await.map_err(|e| {
        ProtocolError::desktop_file_error(format!("Failed to create applications dir: {}", e))
    })?;

    // Generate desktop file content
    let desktop_file_name = get_desktop_file_name(scheme, app_name);
    let desktop_file_path = apps_dir.join(&desktop_file_name);

    let description = options
        .description
        .clone()
        .unwrap_or_else(|| format!("Handle {} URLs", scheme));

    let icon = options
        .icon_path
        .clone()
        .unwrap_or_else(|| app_name.to_lowercase());

    let desktop_content = format!(
        r#"[Desktop Entry]
Version=1.0
Type=Application
Name={app_name}
Comment={description}
Exec="{exe_path}" %u
Icon={icon}
Terminal=false
Categories=Utility;
MimeType=x-scheme-handler/{scheme};
NoDisplay=true
StartupNotify=false
"#,
        app_name = app_name,
        description = description,
        exe_path = exe_path,
        icon = icon,
        scheme = scheme,
    );

    // Write the desktop file
    fs::write(&desktop_file_path, &desktop_content)
        .await
        .map_err(|e| {
            ProtocolError::desktop_file_error(format!("Failed to write desktop file: {}", e))
        })?;

    // Make it executable (not strictly required but good practice)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&desktop_file_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&desktop_file_path, perms);
        }
    }

    // Update desktop database
    let _ = Command::new("update-desktop-database")
        .arg(&apps_dir)
        .output()
        .await;

    // Register with xdg-mime as default handler
    if options.set_as_default.unwrap_or(true) {
        let mime_type = format!("x-scheme-handler/{}", scheme);
        let output = Command::new("xdg-mime")
            .args(["default", &desktop_file_name, &mime_type])
            .output()
            .await
            .map_err(|e| {
                error!("Failed to execute xdg-mime: {}", e);
                ProtocolError::tool_not_found("xdg-mime not available")
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("xdg-mime default failed: {}", stderr);
            // Don't fail - the desktop file is still created
        }
    }

    debug!(scheme = %scheme, desktop_file = %desktop_file_path.display(), "Linux: Protocol registered");

    Ok(RegistrationResult {
        success: true,
        scheme: scheme.to_string(),
        was_already_registered,
        previous_handler,
    })
}

/// Unregister a custom URL protocol handler on Linux
pub async fn unregister_protocol(scheme: &str) -> Result<bool, ProtocolError> {
    debug!(scheme = %scheme, "Linux: unregister_protocol");

    let apps_dir = get_applications_dir();

    // Find and remove desktop files for this scheme
    let mut removed = false;

    if let Ok(mut entries) = fs::read_dir(&apps_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".desktop") && name.contains(&format!("-{}.desktop", scheme)) {
                    // Read and verify it's for this scheme
                    if let Ok(content) = fs::read_to_string(&path).await {
                        if content.contains(&format!("x-scheme-handler/{}", scheme)) {
                            fs::remove_file(&path).await.map_err(|e| {
                                ProtocolError::desktop_file_error(format!(
                                    "Failed to remove desktop file: {}",
                                    e
                                ))
                            })?;
                            removed = true;
                            debug!(path = %path.display(), "Linux: Removed desktop file");
                        }
                    }
                }
            }
        }
    }

    // Update desktop database
    let _ = Command::new("update-desktop-database")
        .arg(&apps_dir)
        .output()
        .await;

    Ok(removed)
}

/// Check if a scheme is registered on Linux
pub async fn is_registered(scheme: &str) -> Result<RegistrationStatus, ProtocolError> {
    debug!(scheme = %scheme, "Linux: is_registered");

    let mime_type = format!("x-scheme-handler/{}", scheme);

    // Query xdg-mime for the default handler
    let output = Command::new("xdg-mime")
        .args(["query", "default", &mime_type])
        .output()
        .await
        .map_err(|e| ProtocolError::tool_not_found(format!("xdg-mime not available: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if stdout.is_empty() {
        return Ok(RegistrationStatus {
            is_registered: false,
            is_default: false,
            registered_by: None,
        });
    }

    // Check if it's our app by looking at the desktop file
    let current_exe = std::env::current_exe().ok();
    let is_default = if let Some(exe) = &current_exe {
        let apps_dir = get_applications_dir();
        let desktop_path = apps_dir.join(&stdout);

        if let Ok(content) = fs::read_to_string(&desktop_path).await {
            content.contains(&exe.to_string_lossy().to_string())
        } else {
            false
        }
    } else {
        false
    };

    // Extract app name from desktop file name
    let registered_by = stdout
        .strip_suffix(".desktop")
        .map(|s| s.to_string())
        .or(Some(stdout.clone()));

    Ok(RegistrationStatus {
        is_registered: true,
        is_default,
        registered_by,
    })
}

/// Set this app as the default handler for a scheme on Linux
pub async fn set_as_default(scheme: &str) -> Result<bool, ProtocolError> {
    debug!(scheme = %scheme, "Linux: set_as_default");

    let exe_path = std::env::current_exe()
        .map_err(|e| ProtocolError::generic(format!("Could not get exe path: {}", e)))?
        .to_string_lossy()
        .to_string();

    let app_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "App".to_string());

    // First ensure we have a desktop file
    let options = RegistrationOptions {
        set_as_default: Some(true),
        ..Default::default()
    };

    let result = register_protocol(scheme, "", &app_name, &exe_path, &options).await?;

    Ok(result.success)
}

/// Check platform capabilities on Linux
pub fn check_capabilities() -> ProtocolCapabilities {
    // Check if xdg-mime is available
    let xdg_mime_available = std::process::Command::new("xdg-mime")
        .arg("--version")
        .output()
        .is_ok();

    // Check if we can write to applications directory
    let apps_dir = get_applications_dir();
    let can_write = if apps_dir.exists() {
        // Try to check write permissions
        std::fs::metadata(&apps_dir)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false)
    } else {
        // Try to create it
        std::fs::create_dir_all(&apps_dir).is_ok()
    };

    ProtocolCapabilities {
        can_register: xdg_mime_available && can_write,
        can_query: xdg_mime_available,
        can_deep_link: xdg_mime_available,
        platform: "linux".to_string(),
        notes: if xdg_mime_available && can_write {
            Some(format!(
                "Using xdg-mime with desktop files in {}",
                apps_dir.display()
            ))
        } else if !xdg_mime_available {
            Some("xdg-mime not available - install xdg-utils".to_string())
        } else {
            Some(format!(
                "Cannot write to applications directory: {}",
                apps_dir.display()
            ))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_desktop_file_name() {
        assert_eq!(
            get_desktop_file_name("myapp", "My App"),
            "my-app-myapp.desktop"
        );
        assert_eq!(
            get_desktop_file_name("coolscheme", "CoolApp"),
            "coolapp-coolscheme.desktop"
        );
    }

    #[test]
    fn test_get_applications_dir() {
        let dir = get_applications_dir();
        assert!(dir.to_string_lossy().contains("applications"));
    }

    #[test]
    fn test_check_capabilities() {
        let caps = check_capabilities();
        assert_eq!(caps.platform, "linux");
    }
}
