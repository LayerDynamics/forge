//! Dock menu model + pure spec-builder.
//!
//! This module has two halves:
//!
//! - A **pure, cross-platform** layer (this file's top half): it normalizes the
//!   loosely-typed [`crate::MenuItem`]s coming from TypeScript into a validated
//!   [`MenuItemSpec`] tree — classifying item kinds, applying defaults, and
//!   parsing accelerators. It touches no AppKit and is exhaustively unit-tested,
//!   so it is the maximally CI-verifiable core of the dock-menu feature.
//! - A **macOS-only** layer ([`mac`], compiled with `#[cfg(target_os = "macos")]`)
//!   that turns a `MenuItemSpec` tree into a live `NSMenu`, injects
//!   `applicationDockMenu:` into the app delegate, and routes clicks back to
//!   Deno. That layer is inherently manual-verification-only (a dock menu only
//!   appears on a real right-click), so the pure layer above carries the tests.

use crate::{DockError, MenuItem};

/// The kind of a normalized menu item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItemKind {
    /// A clickable action item.
    Normal,
    /// A toggleable item that shows a checkmark when `checked`.
    Checkbox,
    /// A non-interactive divider line.
    Separator,
}

/// Parsed modifier flags for a keyboard accelerator. macOS-oriented: `CmdOrCtrl`
/// maps to Command since the dock is macOS-only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifierFlags {
    pub command: bool,
    pub control: bool,
    pub option: bool,
    pub shift: bool,
}

impl ModifierFlags {
    pub fn is_empty(&self) -> bool {
        !(self.command || self.control || self.option || self.shift)
    }
}

/// A parsed keyboard accelerator (e.g. `"CmdOrCtrl+Shift+S"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accelerator {
    pub modifiers: ModifierFlags,
    /// The key equivalent, normalized to a lowercase single character where
    /// applicable (e.g. `"s"`). Stored as-is for named keys.
    pub key: String,
}

/// A validated, normalized menu item ready for NSMenu construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItemSpec {
    /// Stable id reported back when the item is clicked (None for separators or
    /// items the app left unidentified).
    pub id: Option<String>,
    /// Display label (empty for separators).
    pub label: String,
    pub kind: MenuItemKind,
    /// Whether the item is enabled (default true; always false-irrelevant for
    /// separators).
    pub enabled: bool,
    /// Whether a checkbox item shows its checkmark (default false).
    pub checked: bool,
    /// Parsed accelerator, if any.
    pub accelerator: Option<Accelerator>,
    /// Submenu items (empty when the item has no submenu).
    pub submenu: Vec<MenuItemSpec>,
}

/// Normalize and validate a list of raw [`MenuItem`]s into [`MenuItemSpec`]s.
///
/// Errors with [`DockError::InvalidParameter`] if a non-separator item has an
/// empty label (a menu item with no text is almost always a bug) or declares an
/// unknown `type`.
pub fn build_menu_spec(items: &[MenuItem]) -> Result<Vec<MenuItemSpec>, DockError> {
    items.iter().map(build_item_spec).collect()
}

fn build_item_spec(item: &MenuItem) -> Result<MenuItemSpec, DockError> {
    let kind = match item.item_type.as_deref() {
        Some("separator") => MenuItemKind::Separator,
        Some("checkbox") => MenuItemKind::Checkbox,
        Some("normal") | None => MenuItemKind::Normal,
        Some(other) => {
            return Err(DockError::invalid_parameter(format!(
                "unknown menu item type '{}' (expected normal, checkbox, or separator)",
                other
            )));
        }
    };

    if kind == MenuItemKind::Separator {
        return Ok(MenuItemSpec {
            id: item.id.clone(),
            label: String::new(),
            kind,
            enabled: false,
            checked: false,
            accelerator: None,
            submenu: Vec::new(),
        });
    }

    if item.label.trim().is_empty() {
        return Err(DockError::invalid_parameter(
            "menu item label must not be empty (use type: \"separator\" for dividers)",
        ));
    }

    let submenu = match &item.submenu {
        Some(children) => children
            .iter()
            .map(build_item_spec)
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };

    Ok(MenuItemSpec {
        id: item.id.clone(),
        label: item.label.clone(),
        kind,
        enabled: item.enabled.unwrap_or(true),
        checked: item.checked.unwrap_or(false),
        accelerator: item.accelerator.as_deref().and_then(parse_accelerator),
        submenu,
    })
}

/// Parse an accelerator string like `"CmdOrCtrl+Shift+S"` into modifiers + key.
///
/// Tokens are split on `+` and matched case-insensitively. Recognized modifiers:
/// `Cmd`/`Command`/`CmdOrCtrl`/`Super`/`Meta` (Command), `Ctrl`/`Control`,
/// `Alt`/`Option`, `Shift`. The final non-modifier token is the key equivalent
/// (lowercased when a single character). Returns `None` if there is no key part.
pub fn parse_accelerator(input: &str) -> Option<Accelerator> {
    let mut modifiers = ModifierFlags::default();
    let mut key: Option<String> = None;

    for raw in input.split('+') {
        let token = raw.trim();
        if token.is_empty() {
            continue;
        }
        match token.to_ascii_lowercase().as_str() {
            "cmd" | "command" | "cmdorctrl" | "super" | "meta" => modifiers.command = true,
            "ctrl" | "control" => modifiers.control = true,
            "alt" | "option" => modifiers.option = true,
            "shift" => modifiers.shift = true,
            _ => {
                // The last non-modifier token wins as the key.
                key = Some(normalize_key(token));
            }
        }
    }

    key.map(|key| Accelerator { modifiers, key })
}

/// Normalize a key token: single characters lowercase to their NSMenuItem key
/// equivalent; multi-character named keys are kept as written.
fn normalize_key(token: &str) -> String {
    if token.chars().count() == 1 {
        token.to_ascii_lowercase()
    } else {
        token.to_string()
    }
}

/// macOS-only: NSMenu construction, delegate injection, and click routing.
#[cfg(target_os = "macos")]
pub mod mac;

#[cfg(test)]
mod tests {
    use super::*;

    fn item(label: &str) -> MenuItem {
        MenuItem {
            id: None,
            label: label.to_string(),
            accelerator: None,
            enabled: None,
            checked: None,
            submenu: None,
            item_type: None,
        }
    }

    #[test]
    fn normal_item_defaults() {
        let specs = build_menu_spec(&[item("Open")]).unwrap();
        assert_eq!(specs.len(), 1);
        let s = &specs[0];
        assert_eq!(s.label, "Open");
        assert_eq!(s.kind, MenuItemKind::Normal);
        assert!(s.enabled, "items default to enabled");
        assert!(!s.checked);
        assert!(s.accelerator.is_none());
        assert!(s.submenu.is_empty());
    }

    #[test]
    fn separator_ignores_label_and_is_kind_separator() {
        let mut sep = item("ignored");
        sep.item_type = Some("separator".to_string());
        let specs = build_menu_spec(&[sep]).unwrap();
        assert_eq!(specs[0].kind, MenuItemKind::Separator);
        assert_eq!(specs[0].label, "");
    }

    #[test]
    fn checkbox_checked_state_carried() {
        let mut cb = item("Word Wrap");
        cb.item_type = Some("checkbox".to_string());
        cb.checked = Some(true);
        let specs = build_menu_spec(&[cb]).unwrap();
        assert_eq!(specs[0].kind, MenuItemKind::Checkbox);
        assert!(specs[0].checked);
    }

    #[test]
    fn disabled_flag_respected() {
        let mut it = item("Save");
        it.enabled = Some(false);
        let specs = build_menu_spec(&[it]).unwrap();
        assert!(!specs[0].enabled);
    }

    #[test]
    fn empty_label_non_separator_is_error() {
        let err = build_menu_spec(&[item("   ")]).unwrap_err();
        assert!(matches!(err, DockError::InvalidParameter { .. }));
    }

    #[test]
    fn unknown_type_is_error() {
        let mut bad = item("X");
        bad.item_type = Some("radio".to_string());
        let err = build_menu_spec(&[bad]).unwrap_err();
        assert!(matches!(err, DockError::InvalidParameter { .. }));
    }

    #[test]
    fn nested_submenu_is_recursively_built() {
        let mut parent = item("File");
        parent.submenu = Some(vec![item("New"), {
            let mut sep = item("");
            sep.item_type = Some("separator".to_string());
            sep
        }]);
        let specs = build_menu_spec(&[parent]).unwrap();
        assert_eq!(specs[0].submenu.len(), 2);
        assert_eq!(specs[0].submenu[0].label, "New");
        assert_eq!(specs[0].submenu[1].kind, MenuItemKind::Separator);
    }

    #[test]
    fn ids_preserved_for_click_routing() {
        let mut it = item("Preferences");
        it.id = Some("prefs".to_string());
        let specs = build_menu_spec(&[it]).unwrap();
        assert_eq!(specs[0].id.as_deref(), Some("prefs"));
    }

    #[test]
    fn accelerator_cmd_shift_letter() {
        let acc = parse_accelerator("CmdOrCtrl+Shift+S").unwrap();
        assert!(acc.modifiers.command);
        assert!(acc.modifiers.shift);
        assert!(!acc.modifiers.control);
        assert!(!acc.modifiers.option);
        assert_eq!(acc.key, "s");
    }

    #[test]
    fn accelerator_aliases() {
        let acc = parse_accelerator("Command+Option+Q").unwrap();
        assert!(acc.modifiers.command);
        assert!(acc.modifiers.option);
        assert_eq!(acc.key, "q");

        let ctrl = parse_accelerator("Control+A").unwrap();
        assert!(ctrl.modifiers.control);
        assert_eq!(ctrl.key, "a");
    }

    #[test]
    fn accelerator_named_key_kept() {
        let acc = parse_accelerator("Cmd+Enter").unwrap();
        assert!(acc.modifiers.command);
        assert_eq!(acc.key, "Enter");
    }

    #[test]
    fn accelerator_modifiers_only_is_none() {
        assert!(parse_accelerator("Cmd+Shift").is_none());
        assert!(parse_accelerator("").is_none());
    }

    #[test]
    fn accelerator_parsed_into_spec() {
        let mut it = item("Save");
        it.accelerator = Some("CmdOrCtrl+S".to_string());
        let specs = build_menu_spec(&[it]).unwrap();
        let acc = specs[0].accelerator.as_ref().unwrap();
        assert!(acc.modifiers.command);
        assert_eq!(acc.key, "s");
    }
}
