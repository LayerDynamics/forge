//! Forge CLI library surface.
//!
//! This crate is primarily the `forge` binary (`src/main.rs`), but it also
//! exposes its **clap command model** as a library so external tooling can
//! introspect the exact CLI surface instead of re-parsing source.
//!
//! In particular, the documentation drift gate (`forge-docs-check`) calls
//! [`cli()`] to enumerate subcommands, arguments, and options and to generate
//! the CLI reference in `site/src/content/docs/crates/forge.md`. The binary
//! consumes the same [`Cli`]/[`Commands`] model for argument dispatch, so the
//! docs and the runtime parser can never diverge — they are the same model.

use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

/// Extra help shown under the generated command list (kept from the original
/// hand-written usage banner so `forge --help` still points at the examples).
pub const AFTER_HELP: &str = "\
Getting Started:
  Copy an example from the examples/ folder to start a new app:
  - examples/example-deno-app   Minimal TypeScript app
  - examples/react-app          React with TypeScript
  - examples/nextjs-app         Next.js-style patterns
  - examples/svelte-app         Svelte with TypeScript
  - examples/todo-app           Todo app with persistence
  - examples/text-editor        File operations example

Bundle output formats:
  Windows: .msix package
  macOS:   .app bundle + .dmg disk image
  Linux:   .AppImage or .tar.gz";

/// Forge — build cross-platform desktop apps with TypeScript and Deno.
#[derive(Parser)]
#[command(
    name = "forge",
    version,
    about = "Forge — build cross-platform desktop apps with TypeScript and Deno",
    after_help = AFTER_HELP,
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level `forge` subcommands.
#[derive(Subcommand)]
pub enum Commands {
    /// Run an app in development mode (hot reload, full debugging)
    Dev {
        /// App directory (contains manifest.app.toml)
        app_dir: PathBuf,
    },
    /// Build an app's web assets for production
    Build {
        /// App directory
        app_dir: PathBuf,
    },
    /// Package an app into a platform distributable (.app/.dmg, .msix, AppImage)
    Bundle {
        /// App directory
        app_dir: PathBuf,
    },
    /// Ahead-of-time compile an app's TypeScript to JavaScript
    Smelt {
        /// App directory
        app_dir: PathBuf,
        /// Output directory for the compiled JavaScript tree
        #[arg(long, short)]
        out: Option<PathBuf>,
        /// Also write the standalone-binary bootstrap shim
        #[arg(long)]
        embed: bool,
    },
    /// Code-sign a bundled artifact for distribution
    Sign {
        /// Signing identity (e.g. "Developer ID Application: Name (TEAM)")
        #[arg(long, short)]
        identity: Option<String>,
        /// The bundled artifact to sign
        artifact: PathBuf,
    },
    /// Manage app icons
    Icon {
        #[command(subcommand)]
        command: IconCommand,
    },
    /// Generate API documentation from extension TypeScript/Rust source
    #[command(disable_help_flag = true)]
    Docs {
        /// Options/target forwarded to the docs generator: --all-extensions,
        /// --extension <name>, --output <dir>, --format <astro|html|both>
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// `forge icon` subcommands.
#[derive(Subcommand)]
pub enum IconCommand {
    /// Create the default Forge-branded icon at <path>
    Create {
        /// Output path for the icon (PNG)
        path: PathBuf,
    },
    /// Validate an app's icon meets platform requirements
    Validate {
        /// App directory (defaults to the current directory)
        #[arg(default_value = ".")]
        app_dir: PathBuf,
    },
}

/// The fully-built clap command tree.
///
/// This is the single source of truth for the `forge` CLI surface. The
/// documentation generator and drift gate introspect the returned
/// [`clap::Command`] (subcommands, arguments, options, help text) so the
/// published CLI reference always matches the parser the binary runs.
pub fn cli() -> clap::Command {
    Cli::command()
}
