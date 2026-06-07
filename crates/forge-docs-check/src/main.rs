//! `forge-docs-check` binary: run every documentation drift rule and exit
//! non-zero (with a punch-list) if the docs have drifted from the code.

use anyhow::Result;
use forge_docs_check::{apiblock, discovery::Workspace, run_all_checks};
use std::process::ExitCode;

fn main() -> Result<ExitCode> {
    let ws = Workspace::discover()?;

    // `--write-api-blocks`: regenerate every opted-in `<!-- forge:api -->` block
    // in place from the SDK, then exit. This is the marker-hybrid "generator".
    if std::env::args().any(|a| a == "--write-api-blocks") {
        let written = apiblock::write_all(&ws)?;
        if written.is_empty() {
            println!("API signature blocks already up to date.");
        } else {
            println!("Refreshed {} API block(s):", written.len());
            for path in &written {
                println!(
                    "  {}",
                    path.strip_prefix(&ws.root).unwrap_or(path).display()
                );
            }
        }
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
