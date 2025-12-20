//! ext_updater - Application auto-update extension for Forge apps.
//!
//! Provides functionality to check for updates, download, verify, and install
//! application updates. Supports both GitHub Releases and custom JSON manifest formats.
//!
//! Error codes: 5000-5099

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::{op2, Extension, OpState};
use deno_error::JsError;
use forge_weld_macro::{weld_enum, weld_op, weld_struct};
use futures_util::StreamExt;
use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

// ============================================================================
// Error Types (Error codes: 5000-5099)
// ============================================================================

/// Error codes for the updater extension.
#[weld_enum]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UpdaterErrorCode {
    /// Generic updater error (5000)
    Generic = 5000,
    /// Failed to check for updates (5001)
    CheckFailed = 5001,
    /// Failed to download update (5002)
    DownloadFailed = 5002,
    /// Package verification failed (5003)
    VerificationFailed = 5003,
    /// Failed to install update (5004)
    InstallFailed = 5004,
    /// No update available (5005)
    NoUpdate = 5005,
    /// Network error during update operation (5006)
    NetworkError = 5006,
    /// Invalid manifest format (5007)
    InvalidManifest = 5007,
    /// Permission denied (5008)
    PermissionDenied = 5008,
    /// Update already in progress (5009)
    AlreadyInProgress = 5009,
    /// Update operation was cancelled (5010)
    Cancelled = 5010,
    /// Not configured (5011)
    NotConfigured = 5011,
    /// Invalid version format (5012)
    InvalidVersion = 5012,
}

/// Updater extension error type.
#[derive(Debug, Error, JsError)]
pub enum UpdaterError {
    #[error("[{code:?}] {message}")]
    #[class(generic)]
    Generic {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    CheckFailed {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    DownloadFailed {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    VerificationFailed {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    InstallFailed {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    NoUpdate {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    NetworkError {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    InvalidManifest {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    PermissionDenied {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    AlreadyInProgress {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    Cancelled {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    NotConfigured {
        code: UpdaterErrorCode,
        message: String,
    },

    #[error("[{code:?}] {message}")]
    #[class(generic)]
    InvalidVersion {
        code: UpdaterErrorCode,
        message: String,
    },
}

impl UpdaterError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: UpdaterErrorCode::Generic,
            message: message.into(),
        }
    }

    pub fn check_failed(message: impl Into<String>) -> Self {
        Self::CheckFailed {
            code: UpdaterErrorCode::CheckFailed,
            message: message.into(),
        }
    }

    pub fn download_failed(message: impl Into<String>) -> Self {
        Self::DownloadFailed {
            code: UpdaterErrorCode::DownloadFailed,
            message: message.into(),
        }
    }

    pub fn verification_failed(message: impl Into<String>) -> Self {
        Self::VerificationFailed {
            code: UpdaterErrorCode::VerificationFailed,
            message: message.into(),
        }
    }

    pub fn install_failed(message: impl Into<String>) -> Self {
        Self::InstallFailed {
            code: UpdaterErrorCode::InstallFailed,
            message: message.into(),
        }
    }

    pub fn no_update(message: impl Into<String>) -> Self {
        Self::NoUpdate {
            code: UpdaterErrorCode::NoUpdate,
            message: message.into(),
        }
    }

    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError {
            code: UpdaterErrorCode::NetworkError,
            message: message.into(),
        }
    }

    pub fn invalid_manifest(message: impl Into<String>) -> Self {
        Self::InvalidManifest {
            code: UpdaterErrorCode::InvalidManifest,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: UpdaterErrorCode::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn already_in_progress(message: impl Into<String>) -> Self {
        Self::AlreadyInProgress {
            code: UpdaterErrorCode::AlreadyInProgress,
            message: message.into(),
        }
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::Cancelled {
            code: UpdaterErrorCode::Cancelled,
            message: message.into(),
        }
    }

    pub fn not_configured(message: impl Into<String>) -> Self {
        Self::NotConfigured {
            code: UpdaterErrorCode::NotConfigured,
            message: message.into(),
        }
    }

    pub fn invalid_version(message: impl Into<String>) -> Self {
        Self::InvalidVersion {
            code: UpdaterErrorCode::InvalidVersion,
            message: message.into(),
        }
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// Legacy extension info for backward compatibility.
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub status: &'static str,
}

/// Update source configuration.
#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum UpdateSource {
    /// GitHub Releases source
    GitHub { owner: String, repo: String },
    /// Custom JSON manifest URL
    Custom { url: String },
}

/// GitHub release API response structure.
#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    prerelease: bool,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

/// GitHub release asset structure.
#[derive(Debug, Clone, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    content_type: Option<String>,
}

/// Custom manifest format.
#[derive(Debug, Clone, Deserialize)]
struct CustomManifest {
    version: String,
    platforms: HashMap<String, PlatformAsset>,
    release_notes: Option<String>,
    publish_date: Option<String>,
}

/// Platform-specific asset in custom manifest.
#[derive(Debug, Clone, Deserialize)]
struct PlatformAsset {
    url: String,
    sha256: Option<String>,
    size: Option<u64>,
}

/// Update configuration.
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Update source (GitHub or Custom)
    pub source: UpdateSource,
    /// Current application version
    pub current_version: String,
    /// Whether to include prereleases
    pub include_prereleases: bool,
}

/// Information about an available update.
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// New version string
    pub version: String,
    /// Download URL for the current platform
    pub download_url: String,
    /// Release notes (if available)
    pub release_notes: Option<String>,
    /// Download size in bytes
    pub size_bytes: u64,
    /// SHA256 checksum (if available)
    pub sha256: Option<String>,
    /// Publish date (if available)
    pub publish_date: Option<String>,
    /// Whether this is a prerelease
    pub is_prerelease: bool,
    /// All available assets
    pub assets: Vec<UpdateAsset>,
}

/// Individual update asset.
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAsset {
    /// Asset filename
    pub name: String,
    /// Download URL
    pub url: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Content type (MIME)
    pub content_type: Option<String>,
}

/// Download progress information.
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Total bytes to download
    pub total_bytes: u64,
    /// Progress percentage (0-100)
    pub percent: f64,
    /// Current state
    pub state: UpdateState,
}

/// Update state enumeration.
#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    /// No update operation in progress
    Idle,
    /// Checking for updates
    Checking,
    /// Update available, not yet downloaded
    UpdateAvailable,
    /// Downloading update package
    Downloading,
    /// Verifying downloaded package
    Verifying,
    /// Update downloaded and verified, ready to install
    ReadyToInstall,
    /// Installing update
    Installing,
    /// Update complete (app will restart)
    Complete,
    /// Update failed
    Failed,
}

/// Pending update information.
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingUpdate {
    /// Update info
    pub info: UpdateInfo,
    /// Local path to downloaded file
    pub local_path: String,
    /// Whether verification passed
    pub verified: bool,
}

/// Current updater status.
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterStatus {
    /// Current state
    pub state: UpdateState,
    /// Current progress (if downloading)
    pub progress: Option<UpdateProgress>,
    /// Available update info (if any)
    pub available_update: Option<UpdateInfo>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Whether an update source is configured
    pub configured: bool,
}

// ============================================================================
// State Management
// ============================================================================

/// Internal updater state.
#[derive(Debug)]
struct UpdaterStateInner {
    /// Update source configuration
    config: Option<UpdateConfig>,
    /// Current state
    state: UpdateState,
    /// Download progress
    progress: UpdateProgress,
    /// Available update information
    available_update: Option<UpdateInfo>,
    /// Pending downloaded update
    pending_update: Option<PendingUpdate>,
    /// Error message
    error: Option<String>,
    /// Cancel flag
    cancelled: bool,
    /// HTTP client
    client: Client,
}

impl Default for UpdaterStateInner {
    fn default() -> Self {
        Self {
            config: None,
            state: UpdateState::Idle,
            progress: UpdateProgress {
                downloaded_bytes: 0,
                total_bytes: 0,
                percent: 0.0,
                state: UpdateState::Idle,
            },
            available_update: None,
            pending_update: None,
            error: None,
            cancelled: false,
            client: Client::builder()
                .user_agent(concat!("forge-updater/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

/// Thread-safe updater state wrapper.
#[derive(Debug, Clone)]
pub struct UpdaterState {
    inner: Arc<RwLock<UpdaterStateInner>>,
}

impl Default for UpdaterState {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdaterState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(UpdaterStateInner::default())),
        }
    }
}

/// Initialize updater state in OpState.
pub fn init_updater_state(state: &mut OpState) {
    debug!("Initializing updater state");
    state.put(UpdaterState::new());
}

// ============================================================================
// Platform Detection
// ============================================================================

/// Get current platform identifier for asset matching.
fn get_platform_identifier() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // Map to common platform identifiers
    let os_name = match os {
        "macos" => "darwin",
        "windows" => "win32",
        "linux" => "linux",
        other => other,
    };

    let arch_name = match arch {
        "x86_64" => "x64",
        "aarch64" => "aarch64",
        "x86" => "x86",
        "arm" => "arm",
        other => other,
    };

    format!("{}-{}", os_name, arch_name)
}

/// Get file extension for current platform.
fn get_platform_extension() -> &'static str {
    match std::env::consts::OS {
        "macos" => "dmg",
        "windows" => "exe",
        "linux" => "AppImage",
        _ => "tar.gz",
    }
}

/// Check if an asset name matches the current platform.
fn asset_matches_platform(name: &str, app_name: Option<&str>) -> bool {
    let platform = get_platform_identifier();
    let ext = get_platform_extension();
    let name_lower = name.to_lowercase();

    // Check for platform in name
    let has_platform = name_lower.contains(&platform.to_lowercase())
        || (name_lower.contains("darwin") && std::env::consts::OS == "macos")
        || (name_lower.contains("macos") && std::env::consts::OS == "macos")
        || (name_lower.contains("win") && std::env::consts::OS == "windows")
        || (name_lower.contains("linux") && std::env::consts::OS == "linux");

    // Check for architecture
    let has_arch = name_lower.contains("x64")
        || name_lower.contains("x86_64")
        || name_lower.contains("amd64")
        || name_lower.contains("aarch64")
        || name_lower.contains("arm64")
        || name_lower.contains(std::env::consts::ARCH);

    // Check for correct extension
    let has_ext = name_lower.ends_with(ext)
        || name_lower.ends_with(".zip")
        || name_lower.ends_with(".tar.gz")
        || name_lower.ends_with(".dmg")
        || name_lower.ends_with(".app.zip")
        || name_lower.ends_with(".msi")
        || name_lower.ends_with(".msix");

    // If app name provided, check if it matches
    let name_matches = app_name
        .map(|n| name_lower.contains(&n.to_lowercase()))
        .unwrap_or(true);

    has_platform && has_ext && name_matches && has_arch
}

// ============================================================================
// Legacy Operations
// ============================================================================

#[weld_op]
#[op2]
#[serde]
fn op_updater_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_updater",
        version: env!("CARGO_PKG_VERSION"),
        status: "active",
    }
}

#[weld_op]
#[op2]
#[string]
fn op_updater_echo(#[string] message: String) -> String {
    message
}

// ============================================================================
// Configuration Operations
// ============================================================================

/// Configure GitHub Releases as update source.
#[weld_op(async)]
#[op2(async)]
async fn op_updater_configure_github(
    state: Rc<RefCell<OpState>>,
    #[string] owner: String,
    #[string] repo: String,
    #[string] current_version: String,
    include_prereleases: bool,
) -> Result<(), UpdaterError> {
    let updater_state = {
        let s = state.borrow_mut();
        s.borrow::<UpdaterState>().inner.clone()
    };

    let config = UpdateConfig {
        source: UpdateSource::GitHub { owner, repo },
        current_version,
        include_prereleases,
    };

    {
        let mut inner = updater_state.write().await;
        inner.config = Some(config);
        inner.state = UpdateState::Idle;
        inner.error = None;
    }

    info!("Configured GitHub releases update source");
    Ok(())
}

/// Configure custom manifest URL as update source.
#[weld_op(async)]
#[op2(async)]
async fn op_updater_configure_custom(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[string] current_version: String,
    include_prereleases: bool,
) -> Result<(), UpdaterError> {
    // Validate URL
    let _parsed = url::Url::parse(&url)
        .map_err(|e| UpdaterError::invalid_manifest(format!("Invalid URL: {}", e)))?;

    let updater_state = {
        let s = state.borrow_mut();
        s.borrow::<UpdaterState>().inner.clone()
    };

    let config = UpdateConfig {
        source: UpdateSource::Custom { url },
        current_version,
        include_prereleases,
    };

    {
        let mut inner = updater_state.write().await;
        inner.config = Some(config);
        inner.state = UpdateState::Idle;
        inner.error = None;
    }

    info!("Configured custom manifest update source");
    Ok(())
}

// ============================================================================
// Check Operations
// ============================================================================

/// Check for available updates.
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_updater_check(state: Rc<RefCell<OpState>>) -> Result<Option<UpdateInfo>, UpdaterError> {
    let updater_state = {
        let s = state.borrow_mut();
        s.borrow::<UpdaterState>().inner.clone()
    };

    // Get config
    let (config, client) = {
        let inner = updater_state.read().await;
        let config = inner.config.clone().ok_or_else(|| {
            UpdaterError::not_configured("Update source not configured. Call configure_github() or configure_custom() first.")
        })?;
        (config, inner.client.clone())
    };

    // Update state to checking
    {
        let mut inner = updater_state.write().await;
        inner.state = UpdateState::Checking;
        inner.error = None;
        inner.cancelled = false;
    }

    // Perform check based on source type
    let result = match &config.source {
        UpdateSource::GitHub { owner, repo } => {
            check_github_releases(&client, owner, repo, &config).await
        }
        UpdateSource::Custom { url } => check_custom_manifest(&client, url, &config).await,
    };

    // Update state with result
    match &result {
        Ok(Some(update_info)) => {
            let mut inner = updater_state.write().await;
            inner.state = UpdateState::UpdateAvailable;
            inner.available_update = Some(update_info.clone());
            info!("Update available: {}", update_info.version);
        }
        Ok(None) => {
            let mut inner = updater_state.write().await;
            inner.state = UpdateState::Idle;
            inner.available_update = None;
            info!("No update available");
        }
        Err(e) => {
            let mut inner = updater_state.write().await;
            inner.state = UpdateState::Failed;
            inner.error = Some(e.to_string());
            error!("Update check failed: {}", e);
        }
    }

    result
}

/// Check GitHub releases for updates.
async fn check_github_releases(
    client: &Client,
    owner: &str,
    repo: &str,
    config: &UpdateConfig,
) -> Result<Option<UpdateInfo>, UpdaterError> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    debug!("Fetching GitHub release from: {}", url);

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| UpdaterError::network_error(format!("Failed to fetch releases: {}", e)))?;

    if !response.status().is_success() {
        return Err(UpdaterError::check_failed(format!(
            "GitHub API returned status {}",
            response.status()
        )));
    }

    let release: GitHubRelease = response.json().await.map_err(|e| {
        UpdaterError::invalid_manifest(format!("Failed to parse GitHub release: {}", e))
    })?;

    // Skip prereleases unless configured
    if release.prerelease && !config.include_prereleases {
        debug!("Skipping prerelease: {}", release.tag_name);
        return Ok(None);
    }

    // Parse versions
    let release_version_str = release.tag_name.trim_start_matches('v');
    let release_version = Version::parse(release_version_str).map_err(|e| {
        UpdaterError::invalid_version(format!(
            "Invalid release version '{}': {}",
            release.tag_name, e
        ))
    })?;

    let current_version = Version::parse(&config.current_version).map_err(|e| {
        UpdaterError::invalid_version(format!(
            "Invalid current version '{}': {}",
            config.current_version, e
        ))
    })?;

    // Compare versions
    if release_version <= current_version {
        debug!(
            "Current version {} is up to date (latest: {})",
            current_version, release_version
        );
        return Ok(None);
    }

    // Find matching asset for current platform
    let platform = get_platform_identifier();
    let matching_asset = release
        .assets
        .iter()
        .find(|a| asset_matches_platform(&a.name, None));

    let (download_url, size_bytes) = matching_asset
        .map(|a| (a.browser_download_url.clone(), a.size))
        .ok_or_else(|| {
            UpdaterError::check_failed(format!(
                "No compatible asset found for platform {}",
                platform
            ))
        })?;

    // Convert assets
    let assets: Vec<UpdateAsset> = release
        .assets
        .iter()
        .map(|a| UpdateAsset {
            name: a.name.clone(),
            url: a.browser_download_url.clone(),
            size_bytes: a.size,
            content_type: a.content_type.clone(),
        })
        .collect();

    Ok(Some(UpdateInfo {
        version: release_version.to_string(),
        download_url,
        release_notes: release.body,
        size_bytes,
        sha256: None, // GitHub releases don't provide checksums directly
        publish_date: release.published_at,
        is_prerelease: release.prerelease,
        assets,
    }))
}

/// Check custom manifest for updates.
async fn check_custom_manifest(
    client: &Client,
    url: &str,
    config: &UpdateConfig,
) -> Result<Option<UpdateInfo>, UpdaterError> {
    debug!("Fetching custom manifest from: {}", url);

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| UpdaterError::network_error(format!("Failed to fetch manifest: {}", e)))?;

    if !response.status().is_success() {
        return Err(UpdaterError::check_failed(format!(
            "Manifest server returned status {}",
            response.status()
        )));
    }

    let manifest: CustomManifest = response
        .json()
        .await
        .map_err(|e| UpdaterError::invalid_manifest(format!("Failed to parse manifest: {}", e)))?;

    // Parse versions
    let manifest_version = Version::parse(&manifest.version).map_err(|e| {
        UpdaterError::invalid_version(format!(
            "Invalid manifest version '{}': {}",
            manifest.version, e
        ))
    })?;

    let current_version = Version::parse(&config.current_version).map_err(|e| {
        UpdaterError::invalid_version(format!(
            "Invalid current version '{}': {}",
            config.current_version, e
        ))
    })?;

    // Compare versions
    if manifest_version <= current_version {
        debug!(
            "Current version {} is up to date (latest: {})",
            current_version, manifest_version
        );
        return Ok(None);
    }

    // Find platform asset
    let platform = get_platform_identifier();
    let platform_asset = manifest.platforms.get(&platform).ok_or_else(|| {
        UpdaterError::check_failed(format!(
            "No asset found for platform '{}' in manifest",
            platform
        ))
    })?;

    // Build assets list
    let assets: Vec<UpdateAsset> = manifest
        .platforms
        .iter()
        .map(|(platform_name, asset)| UpdateAsset {
            name: format!("{}-{}", manifest.version, platform_name),
            url: asset.url.clone(),
            size_bytes: asset.size.unwrap_or(0),
            content_type: None,
        })
        .collect();

    Ok(Some(UpdateInfo {
        version: manifest_version.to_string(),
        download_url: platform_asset.url.clone(),
        release_notes: manifest.release_notes,
        size_bytes: platform_asset.size.unwrap_or(0),
        sha256: platform_asset.sha256.clone(),
        publish_date: manifest.publish_date,
        is_prerelease: false,
        assets,
    }))
}

// ============================================================================
// Download Operations
// ============================================================================

/// Download the available update.
#[weld_op(async)]
#[op2(async)]
#[string]
async fn op_updater_download(state: Rc<RefCell<OpState>>) -> Result<String, UpdaterError> {
    let updater_state = {
        let s = state.borrow_mut();
        s.borrow::<UpdaterState>().inner.clone()
    };

    // Get update info and client
    let (update_info, client) = {
        let inner = updater_state.read().await;
        let update_info = inner
            .available_update
            .clone()
            .ok_or_else(|| UpdaterError::no_update("No update available. Call check() first."))?;

        // Check if already in progress
        if inner.state == UpdateState::Downloading {
            return Err(UpdaterError::already_in_progress(
                "Download already in progress",
            ));
        }

        (update_info, inner.client.clone())
    };

    // Update state to downloading
    {
        let mut inner = updater_state.write().await;
        inner.state = UpdateState::Downloading;
        inner.cancelled = false;
        inner.progress = UpdateProgress {
            downloaded_bytes: 0,
            total_bytes: update_info.size_bytes,
            percent: 0.0,
            state: UpdateState::Downloading,
        };
    }

    info!("Starting download from: {}", update_info.download_url);

    // Create temp file
    let temp_dir = tempfile::tempdir()
        .map_err(|e| UpdaterError::download_failed(format!("Failed to create temp dir: {}", e)))?;

    let file_name = update_info
        .download_url
        .split('/')
        .last()
        .unwrap_or("update");
    let file_path = temp_dir.path().join(file_name);

    // Start download
    let response = client
        .get(&update_info.download_url)
        .send()
        .await
        .map_err(|e| UpdaterError::network_error(format!("Failed to start download: {}", e)))?;

    if !response.status().is_success() {
        let mut inner = updater_state.write().await;
        inner.state = UpdateState::Failed;
        inner.error = Some(format!("Download failed with status {}", response.status()));
        return Err(UpdaterError::download_failed(format!(
            "Server returned status {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(update_info.size_bytes);

    // Create file
    let mut file = tokio::fs::File::create(&file_path)
        .await
        .map_err(|e| UpdaterError::download_failed(format!("Failed to create file: {}", e)))?;

    // Download with progress
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        // Check for cancellation
        {
            let inner = updater_state.read().await;
            if inner.cancelled {
                return Err(UpdaterError::cancelled("Download cancelled by user"));
            }
        }

        let chunk = chunk
            .map_err(|e| UpdaterError::network_error(format!("Download stream error: {}", e)))?;

        file.write_all(&chunk)
            .await
            .map_err(|e| UpdaterError::download_failed(format!("Failed to write chunk: {}", e)))?;

        downloaded += chunk.len() as u64;

        // Update progress
        {
            let mut inner = updater_state.write().await;
            inner.progress.downloaded_bytes = downloaded;
            inner.progress.total_bytes = total_size;
            inner.progress.percent = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };
        }
    }

    file.flush()
        .await
        .map_err(|e| UpdaterError::download_failed(format!("Failed to flush file: {}", e)))?;

    // Don't let temp_dir drop and delete the file
    let file_path_str = file_path.to_string_lossy().to_string();
    std::mem::forget(temp_dir);

    // Update state
    {
        let mut inner = updater_state.write().await;
        inner.pending_update = Some(PendingUpdate {
            info: update_info,
            local_path: file_path_str.clone(),
            verified: false,
        });
        inner.progress.state = UpdateState::ReadyToInstall;
        inner.state = UpdateState::ReadyToInstall;
    }

    info!("Download complete: {}", file_path_str);
    Ok(file_path_str)
}

/// Get current download progress.
#[weld_op]
#[op2]
#[serde]
fn op_updater_download_progress(state: &mut OpState) -> Result<UpdateProgress, UpdaterError> {
    let updater_state = state.borrow_mut::<UpdaterState>().inner.clone();

    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        let inner = updater_state.read().await;
        Ok(inner.progress.clone())
    })
}

/// Cancel an in-progress download.
#[weld_op(async)]
#[op2(async)]
async fn op_updater_cancel(state: Rc<RefCell<OpState>>) -> Result<(), UpdaterError> {
    let updater_state = {
        let s = state.borrow_mut();
        s.borrow::<UpdaterState>().inner.clone()
    };

    let mut inner = updater_state.write().await;
    if inner.state == UpdateState::Downloading {
        inner.cancelled = true;
        inner.state = UpdateState::Idle;
        info!("Download cancelled");
        Ok(())
    } else {
        Err(UpdaterError::generic("No download in progress to cancel"))
    }
}

// ============================================================================
// Verification Operations
// ============================================================================

/// Verify downloaded update package.
#[weld_op(async)]
#[op2(async)]
async fn op_updater_verify(state: Rc<RefCell<OpState>>) -> Result<bool, UpdaterError> {
    let updater_state = {
        let s = state.borrow_mut();
        s.borrow::<UpdaterState>().inner.clone()
    };

    let pending = {
        let inner = updater_state.read().await;
        inner
            .pending_update
            .clone()
            .ok_or_else(|| UpdaterError::no_update("No pending update. Call download() first."))?
    };

    // Update state
    {
        let mut inner = updater_state.write().await;
        inner.state = UpdateState::Verifying;
    }

    // If no checksum provided, skip verification
    let expected_sha256 = match &pending.info.sha256 {
        Some(hash) => hash.clone(),
        None => {
            warn!("No checksum provided, skipping verification");
            let mut inner = updater_state.write().await;
            if let Some(ref mut pu) = inner.pending_update {
                pu.verified = true;
            }
            inner.state = UpdateState::ReadyToInstall;
            return Ok(true);
        }
    };

    info!("Verifying downloaded file: {}", pending.local_path);

    // Calculate SHA256
    let file_content = tokio::fs::read(&pending.local_path)
        .await
        .map_err(|e| UpdaterError::verification_failed(format!("Failed to read file: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(&file_content);
    let result = hasher.finalize();
    let actual_sha256 = format!("{:x}", result);

    // Compare
    let verified = actual_sha256.eq_ignore_ascii_case(&expected_sha256);

    {
        let mut inner = updater_state.write().await;
        if verified {
            if let Some(ref mut pu) = inner.pending_update {
                pu.verified = true;
            }
            inner.state = UpdateState::ReadyToInstall;
            info!("Verification successful");
        } else {
            inner.state = UpdateState::Failed;
            inner.error = Some(format!(
                "Checksum mismatch: expected {}, got {}",
                expected_sha256, actual_sha256
            ));
            error!(
                "Verification failed: expected {}, got {}",
                expected_sha256, actual_sha256
            );
        }
    }

    if !verified {
        return Err(UpdaterError::verification_failed("Checksum mismatch"));
    }

    Ok(verified)
}

// ============================================================================
// Install Operations
// ============================================================================

/// Install the downloaded update.
#[weld_op(async)]
#[op2(async)]
async fn op_updater_install(state: Rc<RefCell<OpState>>) -> Result<(), UpdaterError> {
    let updater_state = {
        let s = state.borrow_mut();
        s.borrow::<UpdaterState>().inner.clone()
    };

    let pending = {
        let inner = updater_state.read().await;
        inner
            .pending_update
            .clone()
            .ok_or_else(|| UpdaterError::no_update("No pending update. Call download() first."))?
    };

    // Update state
    {
        let mut inner = updater_state.write().await;
        inner.state = UpdateState::Installing;
    }

    info!("Installing update from: {}", pending.local_path);

    // Platform-specific installation
    let result = install_update_platform(&pending.local_path).await;

    match result {
        Ok(()) => {
            let mut inner = updater_state.write().await;
            inner.state = UpdateState::Complete;
            info!("Update installed successfully");
            Ok(())
        }
        Err(e) => {
            let mut inner = updater_state.write().await;
            inner.state = UpdateState::Failed;
            inner.error = Some(e.to_string());
            Err(e)
        }
    }
}

/// Platform-specific update installation.
async fn install_update_platform(file_path: &str) -> Result<(), UpdaterError> {
    let path = PathBuf::from(file_path);

    match std::env::consts::OS {
        "macos" => install_macos(&path).await,
        "windows" => install_windows(&path).await,
        "linux" => install_linux(&path).await,
        other => Err(UpdaterError::install_failed(format!(
            "Unsupported platform: {}",
            other
        ))),
    }
}

/// Install update on macOS.
async fn install_macos(path: &PathBuf) -> Result<(), UpdaterError> {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "dmg" => {
            // Mount DMG and copy app
            info!("Opening DMG for installation: {:?}", path);

            // Open the DMG - user will manually install
            tokio::process::Command::new("open")
                .arg(path)
                .status()
                .await
                .map_err(|e| UpdaterError::install_failed(format!("Failed to open DMG: {}", e)))?;

            Ok(())
        }
        "zip" => {
            // Unzip and replace app
            info!("Extracting ZIP for installation: {:?}", path);

            let dest_dir = path.parent().unwrap_or(path);
            tokio::process::Command::new("unzip")
                .arg("-o")
                .arg(path)
                .arg("-d")
                .arg(dest_dir)
                .status()
                .await
                .map_err(|e| UpdaterError::install_failed(format!("Failed to unzip: {}", e)))?;

            Ok(())
        }
        _ => Err(UpdaterError::install_failed(format!(
            "Unsupported file type: {}",
            extension
        ))),
    }
}

/// Install update on Windows.
async fn install_windows(path: &PathBuf) -> Result<(), UpdaterError> {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "exe" | "msi" | "msix" => {
            info!("Launching installer: {:?}", path);

            // Launch installer
            tokio::process::Command::new(path).spawn().map_err(|e| {
                UpdaterError::install_failed(format!("Failed to launch installer: {}", e))
            })?;

            Ok(())
        }
        _ => Err(UpdaterError::install_failed(format!(
            "Unsupported file type: {}",
            extension
        ))),
    }
}

/// Install update on Linux.
async fn install_linux(path: &PathBuf) -> Result<(), UpdaterError> {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "AppImage" => {
            info!("Making AppImage executable: {:?}", path);

            // Make executable
            tokio::process::Command::new("chmod")
                .arg("+x")
                .arg(path)
                .status()
                .await
                .map_err(|e| {
                    UpdaterError::install_failed(format!("Failed to chmod AppImage: {}", e))
                })?;

            // Run the new AppImage
            tokio::process::Command::new(path).spawn().map_err(|e| {
                UpdaterError::install_failed(format!("Failed to launch AppImage: {}", e))
            })?;

            Ok(())
        }
        "deb" => {
            info!("Installing .deb package: {:?}", path);

            tokio::process::Command::new("sudo")
                .arg("dpkg")
                .arg("-i")
                .arg(path)
                .status()
                .await
                .map_err(|e| {
                    UpdaterError::install_failed(format!("Failed to install deb: {}", e))
                })?;

            Ok(())
        }
        "rpm" => {
            info!("Installing .rpm package: {:?}", path);

            tokio::process::Command::new("sudo")
                .arg("rpm")
                .arg("-U")
                .arg(path)
                .status()
                .await
                .map_err(|e| {
                    UpdaterError::install_failed(format!("Failed to install rpm: {}", e))
                })?;

            Ok(())
        }
        _ => Err(UpdaterError::install_failed(format!(
            "Unsupported file type: {}",
            extension
        ))),
    }
}

// ============================================================================
// Status Operations
// ============================================================================

/// Get current updater status.
#[weld_op]
#[op2]
#[serde]
fn op_updater_status(state: &mut OpState) -> UpdaterStatus {
    let updater_state = state.borrow_mut::<UpdaterState>().inner.clone();

    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        let inner = updater_state.read().await;
        UpdaterStatus {
            state: inner.state.clone(),
            progress: if inner.state == UpdateState::Downloading {
                Some(inner.progress.clone())
            } else {
                None
            },
            available_update: inner.available_update.clone(),
            error: inner.error.clone(),
            configured: inner.config.is_some(),
        }
    })
}

/// Get current application version.
#[weld_op]
#[op2]
#[string]
fn op_updater_get_current_version(state: &mut OpState) -> Result<String, UpdaterError> {
    let updater_state = state.borrow_mut::<UpdaterState>().inner.clone();

    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        let inner = updater_state.read().await;
        inner
            .config
            .as_ref()
            .map(|c| c.current_version.clone())
            .ok_or_else(|| UpdaterError::not_configured("Update source not configured"))
    })
}

/// Get pending update information.
#[weld_op]
#[op2]
#[serde]
fn op_updater_get_pending_update(state: &mut OpState) -> Option<PendingUpdate> {
    let updater_state = state.borrow_mut::<UpdaterState>().inner.clone();

    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        let inner = updater_state.read().await;
        inner.pending_update.clone()
    })
}

// ============================================================================
// Extension Registration
// ============================================================================

include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn updater_extension() -> Extension {
    runtime_updater::ext()
}
