//! `forge-docs-check` binary: run every documentation drift rule and exit
//! non-zero (with a punch-list) if the docs have drifted from the code.

use anyhow::Result;
use forge_docs_check::{apiblock, checks, discovery::Workspace, run_all_checks};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn report_written(kind: &str, written: &[PathBuf], root: &Path) {
    if written.is_empty() {
        println!("{kind}s already up to date.");
    } else {
        println!("Refreshed {} {kind}(s):", written.len());
        for path in written {
            println!("  {}", path.strip_prefix(root).unwrap_or(path).display());
        }
    }
}

fn main() -> Result<ExitCode> {
    let ws = Workspace::discover()?;

    // `--write-api-blocks`: regenerate every opted-in `<!-- forge:api -->` block
    // in place from the SDK, then exit. This is the marker-hybrid "generator".
    if std::env::args().any(|a| a == "--write-api-blocks") {
        let written = apiblock::write_all(&ws)?;
        report_written("API signature block", &written, &ws.root);
        return Ok(ExitCode::SUCCESS);
    }

    // `--write-counts`: regenerate every `<!-- forge:count:* -->` marker in place
    // from the derived workspace counts, then exit.
    if std::env::args().any(|a| a == "--write-counts") {
        let written = checks::counts::write_counts(&ws)?;
        report_written("count marker", &written, &ws.root);
        return Ok(ExitCode::SUCCESS);
    }

    // `--write-crate-pages`: generate a page for any crate that lacks one (from
    // its //! doc + Cargo.toml), never overwriting an existing page, then exit.
    if std::env::args().any(|a| a == "--write-crate-pages") {
        let written = forge_docs_check::cratepage::write_missing(&ws)?;
        report_written("crate page", &written, &ws.root);
        return Ok(ExitCode::SUCCESS);
    }

    // `--write-example-blocks`: regenerate every `<!-- forge:example -->` block
    // in place from each example app's runtime:* imports, then exit.
    if std::env::args().any(|a| a == "--write-example-blocks") {
        let written = forge_docs_check::exampleblock::write_all(&ws)?;
        report_written("example block", &written, &ws.root);
        return Ok(ExitCode::SUCCESS);
    }

    // `--write-cli`: regenerate the `<!-- forge:cli -->` CLI reference in
    // crates/forge.md from the forge_cli clap model, then exit.
    if std::env::args().any(|a| a == "--write-cli") {
        let written = forge_docs_check::clidoc::write_all(&ws)?;
        report_written("CLI reference", &written, &ws.root);
        return Ok(ExitCode::SUCCESS);
    }

    let report = run_all_checks(&ws);

    println!("{}", report.render());

    if report.is_clean() {
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!(
            "\n{} documentation drift issue(s) found. Run the generators (Phase 2+) or update the docs, then re-run.",
            report.findings.len()
        );
        Ok(ExitCode::FAILURE)
    }
}
