//! Rule `crate-page`: every workspace crate has a documentation page.
//!
//! Caught the `ext_console` and `forge-smelt` gaps in the `Site.md` audit.

use crate::discovery::Workspace;
use crate::Finding;

pub fn check(ws: &Workspace) -> Vec<Finding> {
    let crates_docs = ws.docs_dir().join("crates");
    let mut findings = Vec::new();
    for krate in &ws.crates {
        let stem = krate.crate_page_stem();
        let page = crates_docs.join(format!("{stem}.md"));
        if !page.exists() {
            findings.push(Finding::new(
                "crate-page",
                format!(
                    "crate `{}` has no documentation page (expected site/src/content/docs/crates/{}.md)",
                    krate.dir_name, stem
                ),
            ));
        }
    }
    findings
}
