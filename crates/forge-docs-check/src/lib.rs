//! # forge-docs-check
//!
//! Fails the build when the documentation site (`site/src/content/docs/`) drifts
//! from the code. It mechanizes the audit that produced `Site.md`: missing crate
//! pages, API method drift (in both directions), stale counts, missing CLI
//! commands, and the self-consistency of `forge docs`' own extension list.
//!
//! The same logic runs three ways:
//! - as the `forge-docs-check` binary (prints a punch-list, exits non-zero on drift),
//! - as the `docs_in_sync` integration test (`cargo test` fails on drift),
//! - invoked from CI and the pre-commit hook.
//!
//! Every expectation is derived from the filesystem via [`discovery::Workspace`],
//! so the checker never carries a hand-maintained list that could itself rot.

pub mod apiblock;
pub mod checks;
pub mod clidoc;
pub mod cratepage;
pub mod discovery;
pub mod exampleblock;
pub mod extindex;
pub mod markers;

use discovery::Workspace;

/// A single drift problem, attributed to the rule that found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Stable rule identifier, e.g. `"crate-page"`, `"api-drift"`.
    pub rule: &'static str,
    /// Human-readable, actionable description of the drift.
    pub message: String,
}

impl Finding {
    pub fn new(rule: &'static str, message: impl Into<String>) -> Self {
        Self {
            rule,
            message: message.into(),
        }
    }
}

/// The aggregate result of running every drift rule.
#[derive(Debug, Default, Clone)]
pub struct Report {
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn extend(&mut self, more: impl IntoIterator<Item = Finding>) {
        self.findings.extend(more);
    }

    /// Render the punch-list grouped by rule, in the style of `Site.md`.
    pub fn render(&self) -> String {
        if self.is_clean() {
            return "Documentation is in sync with the code. No drift found.".to_string();
        }
        let mut out = format!(
            "Documentation drift detected ({} issue{}):\n",
            self.findings.len(),
            if self.findings.len() == 1 { "" } else { "s" }
        );
        // Stable, grouped ordering so CI output and diffs are deterministic.
        let mut rules: Vec<&'static str> = self.findings.iter().map(|f| f.rule).collect();
        rules.sort_unstable();
        rules.dedup();
        for rule in rules {
            out.push_str(&format!("\n[{rule}]\n"));
            for f in self.findings.iter().filter(|f| f.rule == rule) {
                out.push_str(&format!("  - {}\n", f.message));
            }
        }
        out
    }
}

/// Run every drift rule against the given workspace and aggregate the findings.
///
/// Rules are intentionally independent: each derives its own expectations from
/// the [`Workspace`] inventory and contributes [`Finding`]s. Adding a rule is a
/// one-line registration here.
pub fn run_all_checks(ws: &Workspace) -> Report {
    let mut report = Report::default();
    report.extend(checks::crate_pages::check(ws));
    report.extend(checks::api_drift::check(ws));
    report.extend(checks::counts::check(ws));
    report.extend(checks::cli_commands::check(ws));
    report.extend(checks::forge_docs::check(ws));
    report.extend(apiblock::check(ws));
    report.extend(exampleblock::check(ws));
    report.extend(extindex::check(ws));
    report.extend(clidoc::check(ws));
    report
}
