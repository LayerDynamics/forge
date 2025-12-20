//! # `runtime:path` - Path Manipulation Extension
//!
//! Cross-platform path manipulation utilities for Forge applications.
//!
//! ## Overview
//!
//! This extension provides pure string-based path manipulation functions that work
//! consistently across all platforms. Operations automatically use the correct path
//! separators (forward slashes on Unix, backslashes on Windows) and handle edge cases
//! like empty paths, missing components, and hidden files.
//!
//! **Key Features:**
//! - **Join Segments**: Combine path components with platform-appropriate separators
//! - **Extract Components**: Get directory names, basenames, and extensions
//! - **Parse Paths**: Split paths into structured components
//! - **Cross-Platform**: Consistent behavior on Unix and Windows
//! - **Pure Functions**: No filesystem access, no permissions required
//!
//! ## API Categories
//!
//! ### Path Construction
//! - `join()` - Combine path segments into a single path
//!
//! ### Path Extraction
//! - `dirname()` - Get directory portion of a path
//! - `basename()` - Get filename portion of a path
//! - `extname()` - Get file extension with leading dot
//!
//! ### Path Parsing
//! - `parts()` - Parse path into directory, basename, and extension
//!
//! ## TypeScript Usage Examples
//!
//! ### Basic Path Manipulation
//! ```typescript
//! import { join, dirname, basename, extname } from "runtime:path";
//!
//! // Join path segments
//! const configPath = join("./data", "config.json");
//! // Unix: "./data/config.json"
//! // Windows: ".\\data\\config.json"
//!
//! // Extract components
//! const dir = dirname("/usr/local/bin/node");   // "/usr/local/bin"
//! const file = basename("/usr/local/bin/node"); // "node"
//! const ext = extname("file.txt");              // ".txt"
//! ```
//!
//! ### Building File Paths
//! ```typescript
//! import { join } from "runtime:path";
//!
//! // Build nested directory paths
//! const imagePath = join("./assets", "images", "logo.png");
//! // Unix: "./assets/images/logo.png"
//! // Windows: ".\\assets\\images\\logo.png"
//!
//! // Join with absolute paths
//! const binPath = join("/usr", "local", "bin", "node");
//! // Unix: "/usr/local/bin/node"
//! ```
//!
//! ### Extracting Path Components
//! ```typescript
//! import { dirname, basename, extname } from "runtime:path";
//!
//! const fullPath = "./data/logs/app.log";
//!
//! const dir = dirname(fullPath);   // "./data/logs"
//! const file = basename(fullPath); // "app.log"
//! const ext = extname(fullPath);   // ".log"
//! ```
//!
//! ### Parsing Complete Paths
//! ```typescript
//! import { parts } from "runtime:path";
//!
//! const p = parts("./data/config.json");
//! console.log(p.dir);  // "./data"
//! console.log(p.base); // "config.json"
//! console.log(p.ext);  // ".json"
//! ```
//!
//! ### Building Modified Paths
//! ```typescript
//! import { parts, join } from "runtime:path";
//!
//! // Create thumbnail from image path
//! const original = "./images/photo.jpg";
//! const p = parts(original);
//! const thumbnail = join(p.dir, `thumb_${p.base}`);
//! console.log(thumbnail); // "./images/thumb_photo.jpg"
//! ```
//!
//! ## Edge Cases
//!
//! ### Empty Results
//! Functions return empty strings when components don't exist:
//! ```typescript
//! dirname("file.txt");     // "" (no directory)
//! extname("README");       // "" (no extension)
//! basename("/path/to/");   // "" (ends with separator)
//! ```
//!
//! ### Hidden Files
//! Dot prefixes are not treated as extensions:
//! ```typescript
//! extname(".gitignore");   // "" (dot prefix, not extension)
//! extname(".config.json"); // ".json" (has real extension)
//! ```
//!
//! ### Multiple Extensions
//! Only the last extension is extracted:
//! ```typescript
//! extname("archive.tar.gz"); // ".gz" (not ".tar.gz")
//! ```
//!
//! ## Platform Support
//!
//! ### Cross-Platform Behavior
//! - **Unix (macOS/Linux)**: Uses forward slashes (`/`) as path separators
//! - **Windows**: Uses backslashes (`\`) as path separators
//! - Operations automatically adapt to the platform at runtime
//!
//! ### Path Separator Handling
//! The functions use Rust's `std::path::Path` which automatically handles platform
//! differences. When constructing paths in TypeScript code, you can use forward
//! slashes everywhere - they will be converted to backslashes on Windows.
//!
//! ### Empty Path Components
//! - `dirname()`, `basename()`, and `extname()` return empty strings when the
//!   requested component doesn't exist
//! - `parts()` returns empty strings for missing components in the struct
//! - Functions never return `null` or throw errors
//!
//! ## No Permissions Required
//!
//! Unlike filesystem operations (`runtime:fs`), path manipulation functions:
//! - Don't access the filesystem
//! - Don't require permissions in `manifest.app.toml`
//! - Work with any path string, whether it exists or not
//! - Are pure string transformations
//!
//! ## Implementation Details
//!
//! ### No State
//! This extension is stateless - all functions are pure transformations that don't
//! maintain any state in `OpState`.
//!
//! ### Rust Standard Library
//! Uses Rust's `std::path::Path` and `std::path::PathBuf` for all operations,
//! ensuring correct platform-specific behavior.
//!
//! ### String Conversion
//! Paths are converted to UTF-8 strings using `to_string_lossy()`, which replaces
//! invalid UTF-8 sequences with replacement characters. This ensures operations
//! never fail on unusual filenames.
//!
//! ## See Also
//!
//! - [`ext_fs`](../ext_fs/index.html) - Filesystem operations (read, write, stat)
//! - [`ext_process`](../ext_process/index.html) - Process spawning with working directories

use deno_core::{op2, Extension};
use forge_weld_macro::{weld_op, weld_struct};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[weld_struct]
#[derive(Serialize)]
struct PathParts {
    dir: String,
    base: String,
    ext: String,
}

fn to_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn path_join(base: String, segments: Vec<String>) -> String {
    let mut pb = PathBuf::from(base);
    for seg in segments {
        pb.push(seg);
    }
    to_string(pb)
}

fn path_dirname(path: &str) -> String {
    Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn path_basename(path: &str) -> String {
    Path::new(&path)
        .file_name()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn path_extname(path: &str) -> String {
    Path::new(&path)
        .extension()
        .map(|p| format!(".{}", p.to_string_lossy()))
        .unwrap_or_default()
}

#[weld_op]
#[op2]
#[string]
fn op_path_join(#[string] base: String, #[serde] segments: Vec<String>) -> String {
    path_join(base, segments)
}

#[weld_op]
#[op2]
#[string]
fn op_path_dirname(#[string] path: String) -> String {
    path_dirname(&path)
}

#[weld_op]
#[op2]
#[string]
fn op_path_basename(#[string] path: String) -> String {
    path_basename(&path)
}

#[weld_op]
#[op2]
#[string]
fn op_path_extname(#[string] path: String) -> String {
    path_extname(&path)
}

#[weld_op]
#[op2]
#[serde]
fn op_path_parts(#[string] path: String) -> PathParts {
    PathParts {
        dir: path_dirname(&path),
        base: path_basename(&path),
        ext: path_extname(&path),
    }
}

deno_core::extension!(
    ext_path,
    ops = [
        op_path_join,
        op_path_dirname,
        op_path_basename,
        op_path_extname,
        op_path_parts
    ]
);

pub fn path_extension() -> Extension {
    ext_path::init()
}
