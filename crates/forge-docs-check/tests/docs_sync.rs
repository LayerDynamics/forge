//! Real-tree drift gate. Runs every rule against the actual repository and
//! compares the findings to a recorded baseline (`tests/known_drift_baseline.txt`).
//!
//! Behaviour:
//! - **New drift** (a finding not in the baseline) fails the test — this is the
//!   enforcement: a PR that adds an undocumented export, a new crate without a
//!   page, a stale count, etc. cannot merge.
//! - **Fixed drift** (a baseline entry that no longer occurs) ALSO fails, with an
//!   instruction to delete it from the baseline. This ratchets the known backlog
//!   down to empty as Phases 2–5 land; once the file is empty the gate enforces
//!   zero drift outright.
//!
//! To re-record the baseline after intentionally changing the docs/code, run:
//!   `UPDATE_DOCS_BASELINE=1 cargo test -p forge-docs-check --test docs_sync`

use forge_docs_check::{discovery::Workspace, run_all_checks};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/known_drift_baseline.txt")
}

/// Current findings as stable, sorted `rule<TAB>message` lines.
fn current_lines() -> Vec<String> {
    let ws = Workspace::discover().expect("discover workspace root");
    let report = run_all_checks(&ws);
    let mut lines: Vec<String> = report
        .findings
        .iter()
        .map(|f| format!("{}\t{}", f.rule, f.message))
        .collect();
    lines.sort();
    lines
}

#[test]
fn docs_in_sync() {
    let lines = current_lines();
    let path = baseline_path();

    if std::env::var("UPDATE_DOCS_BASELINE").is_ok() {
        let body = if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n") + "\n"
        };
        fs::write(&path, body).expect("write baseline");
        eprintln!(
            "Updated baseline: {} entr{} -> {}",
            lines.len(),
            if lines.len() == 1 { "y" } else { "ies" },
            path.display()
        );
        return;
    }

    let baseline_raw = fs::read_to_string(&path).unwrap_or_default();
    let baseline: BTreeSet<String> = baseline_raw
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    let current: BTreeSet<String> = lines.iter().cloned().collect();

    let new_drift: Vec<&String> = current.difference(&baseline).collect();
    let fixed: Vec<&String> = baseline.difference(&current).collect();

    if new_drift.is_empty() && fixed.is_empty() {
        return;
    }

    let mut msg = String::from("\nDocumentation drift gate failed.\n");
    if !new_drift.is_empty() {
        msg.push_str(&format!(
            "\nNEW drift ({}) — fix the docs (or run the Phase 2+ generators):\n",
            new_drift.len()
        ));
        for l in &new_drift {
            msg.push_str(&format!("  + {l}\n"));
        }
    }
    if !fixed.is_empty() {
        msg.push_str(&format!(
            "\nFIXED drift ({}) — remove these stale lines from tests/known_drift_baseline.txt\n(or re-record with UPDATE_DOCS_BASELINE=1):\n",
            fixed.len()
        ));
        for l in &fixed {
            msg.push_str(&format!("  - {l}\n"));
        }
    }
    panic!("{msg}");
}
