//! Drift rules. Each submodule exposes `check(&Workspace) -> Vec<Finding>` and
//! derives its expectations purely from the workspace inventory + the docs tree.

pub mod api_drift;
pub mod cli_commands;
pub mod counts;
pub mod crate_pages;
pub mod forge_docs;
pub mod slug_prefix;

use std::path::Path;

/// Generic hook/handler plumbing emitted into every extensibility-enabled SDK
/// module. These are not part of a module's documented public surface, so the
/// drift rules and the API-signature block both exclude them.
pub(crate) const HOOK_PLUMBING: &[&str] = &[
    "invokeHandler",
    "hasHandler",
    "listHandlers",
    "onAfter",
    "onBefore",
    "onError",
    "registerHandler",
    "removeAllHooks",
    "removeHandler",
];

/// Read a file to a string, returning `None` (not an error) when it is absent.
/// Rules treat "file missing" as a specific [`crate::Finding`], so they need to
/// distinguish absence from an unreadable file explicitly.
pub(crate) fn read_optional(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}
