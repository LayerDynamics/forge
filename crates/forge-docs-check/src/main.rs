//! `forge-docs-check` binary: run every documentation drift rule and exit
//! non-zero (with a punch-list) if the docs have drifted from the code.

use anyhow::Result;
use forge_docs_check::{discovery::Workspace, run_all_checks};
use std::process::ExitCode;

fn main() -> Result<ExitCode> {
    let ws = Workspace::discover()?;
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
