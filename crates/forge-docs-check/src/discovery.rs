//! Inventory of the workspace, derived entirely from the filesystem.
//!
//! Every drift rule expresses its expectation in terms of what *actually exists*
//! in the repository — never a hand-maintained list. The two authoritative roots
//! are the workspace `Cargo.toml` `members` array and the `sdk/` directory of
//! generated TypeScript. If a new crate or runtime module is added, it shows up
//! here automatically, which is the whole point: the checker cannot itself go
//! stale the way the hardcoded `EXTENSIONS` array in `forge_cli/src/docs.rs` did.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

/// A workspace crate and the documentation page it is expected to own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateInfo {
    /// Directory name under `crates/` (e.g. `ext_console`, `forge_cli`).
    pub dir_name: String,
    /// `[package] name` from the crate's `Cargo.toml`.
    pub package_name: String,
    /// Absolute path to the crate directory.
    pub path: PathBuf,
}

impl CrateInfo {
    /// True for runtime extension crates (`crates/ext_*`).
    pub fn is_extension(&self) -> bool {
        self.dir_name.starts_with("ext_")
    }

    /// The crate-doc page stem the site uses under `docs/crates/`.
    ///
    /// The site mirrors directory names with dashes (`ext_image_tools` ->
    /// `ext-image-tools`), with one special case: the CLI crate `forge_cli`
    /// is documented as `forge.md`.
    pub fn crate_page_stem(&self) -> String {
        let dashed = self.dir_name.replace('_', "-");
        if dashed == "forge-cli" {
            "forge".to_string()
        } else {
            dashed
        }
    }
}

/// A generated runtime SDK module (`sdk/runtime.<module>.ts`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdkModule {
    /// Module name as it appears in the specifier, e.g. `fs`, `image_tools`.
    /// Underscores are preserved (API pages keep them: `runtime-image_tools.md`).
    pub name: String,
    /// Absolute path to the `.ts` file.
    pub path: PathBuf,
}

impl SdkModule {
    /// The API-reference page stem the site uses under `docs/api/`
    /// (e.g. `runtime-image_tools`).
    pub fn api_page_stem(&self) -> String {
        format!("runtime-{}", self.name)
    }
}

/// The fully-discovered repository layout shared by all checks.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub crates: Vec<CrateInfo>,
    pub sdk_modules: Vec<SdkModule>,
}

impl Workspace {
    /// Discover the workspace starting from the compiled crate location (works
    /// under both `cargo run` and `cargo test`) falling back to the current dir
    /// (for an installed binary).
    pub fn discover() -> Result<Self> {
        let start = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .or_else(|_| std::env::current_dir())
            .context("failed to determine a starting directory for discovery")?;
        let root = find_workspace_root(&start)?;
        Self::discover_at(&root)
    }

    /// Discover against an explicit workspace root (used by tests with fixtures).
    pub fn discover_at(root: &Path) -> Result<Self> {
        let crates = discover_crates(root)?;
        let sdk_modules = discover_sdk_modules(root)?;
        Ok(Self {
            root: root.to_path_buf(),
            crates,
            sdk_modules,
        })
    }

    /// Extension crates only (`crates/ext_*`), sorted by directory name.
    pub fn extension_crates(&self) -> Vec<&CrateInfo> {
        self.crates.iter().filter(|c| c.is_extension()).collect()
    }

    /// Path to `site/src/content/docs`.
    pub fn docs_dir(&self) -> PathBuf {
        self.root.join("site/src/content/docs")
    }
}

/// Walk up from `start` until a `Cargo.toml` declaring `[workspace]` is found.
pub fn find_workspace_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();
            if content.contains("[workspace]") {
                return Ok(current);
            }
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                return Err(anyhow!(
                    "could not locate a workspace Cargo.toml above {}",
                    start.display()
                ))
            }
        }
    }
}

/// Read the workspace `members` and resolve each into a [`CrateInfo`].
fn discover_crates(root: &Path) -> Result<Vec<CrateInfo>> {
    let cargo_path = root.join("Cargo.toml");
    let raw = std::fs::read_to_string(&cargo_path)
        .with_context(|| format!("reading {}", cargo_path.display()))?;
    let manifest: toml::Value =
        toml::from_str(&raw).with_context(|| format!("parsing {}", cargo_path.display()))?;

    let members = manifest
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .ok_or_else(|| anyhow!("no [workspace].members array in {}", cargo_path.display()))?;

    let mut crates = Vec::with_capacity(members.len());
    for member in members {
        let rel = member
            .as_str()
            .ok_or_else(|| anyhow!("non-string member entry in workspace members"))?;
        let path = root.join(rel);
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("member path has no directory name: {rel}"))?
            .to_string();
        let package_name = read_package_name(&path)
            .with_context(|| format!("reading package name for member {rel}"))?;
        crates.push(CrateInfo {
            dir_name,
            package_name,
            path,
        });
    }
    crates.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    Ok(crates)
}

/// Extract `[package] name` from a crate's `Cargo.toml`.
fn read_package_name(crate_dir: &Path) -> Result<String> {
    let cargo_path = crate_dir.join("Cargo.toml");
    let raw = std::fs::read_to_string(&cargo_path)
        .with_context(|| format!("reading {}", cargo_path.display()))?;
    let manifest: toml::Value = toml::from_str(&raw)?;
    manifest
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("no [package].name in {}", cargo_path.display()))
}

/// List `sdk/runtime.<module>.ts` files as [`SdkModule`]s, sorted by name.
fn discover_sdk_modules(root: &Path) -> Result<Vec<SdkModule>> {
    let sdk_dir = root.join("sdk");
    let mut modules = Vec::new();
    let entries = std::fs::read_dir(&sdk_dir)
        .with_context(|| format!("reading sdk directory {}", sdk_dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(n) => n,
            None => continue,
        };
        // Match `runtime.<module>.ts`, capturing `<module>` (which may contain
        // underscores, e.g. `runtime.image_tools.ts`).
        if let Some(rest) = name.strip_prefix("runtime.") {
            if let Some(module) = rest.strip_suffix(".ts") {
                modules.push(SdkModule {
                    name: module.to_string(),
                    path: entry.path(),
                });
            }
        }
    }
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(modules)
}
