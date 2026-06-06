//! macOS NSMenu construction, dock-menu delegate injection, and click routing.
//!
//! **Manual-verification only.** A dock menu only appears on a real right-click
//! of the running app's dock icon, and clicks only fire through a live
//! NSApplication, so none of this module is exercisable in CI. The
//! CI-verifiable logic (item normalization, accelerator parsing) lives in the
//! pure spec-builder in the parent module.
//!
//! ## How the dock menu is delivered
//!
//! macOS asks the `NSApplication` delegate for `applicationDockMenu:` when the
//! dock icon is right-clicked — there is no `setDockMenu` setter. tao owns the
//! delegate, so we surgically add a single `applicationDockMenu:` method to the
//! delegate's existing class via the Obj-C runtime (`class_addMethod`), leaving
//! every other delegate method untouched. The added method reads the current
//! menu **lazily** from a global at click time, so `setMenu` only has to update
//! that global — delegate-install ordering can never break it.
//!
//! ## Manual verification procedure
//!
//! Because the dock menu only renders on a real right-click and clicks only fire
//! through a running NSApplication, verify by hand on macOS:
//!
//! 1. In an example app's `src/main.ts`, create a window, then:
//!    ```ts
//!    import { setMenu, onMenuItemClick } from "runtime:dock";
//!    setMenu([
//!      { id: "new", label: "New Window", accelerator: "CmdOrCtrl+N" },
//!      { type: "separator" },
//!      { id: "wrap", label: "Word Wrap", type: "checkbox", checked: true },
//!      { label: "More", submenu: [{ id: "about", label: "About" }] },
//!      { id: "disabled", label: "Unavailable", enabled: false },
//!    ]);
//!    onMenuItemClick((id) => console.log("dock menu click:", id));
//!    ```
//! 2. `cargo run -p forge_cli -- dev <app>`.
//! 3. Right-click the app's dock icon. Confirm: the five items render with the
//!    separator, the "Word Wrap" checkmark, the "More" submenu arrow, and the
//!    greyed-out "Unavailable" item.
//! 4. Click "New Window", "About" (in the submenu), and "Word Wrap"; confirm the
//!    console logs `new`, `about`, `wrap`. Confirm "Unavailable" cannot be
//!    clicked.
//! 5. Call `setMenu` again with a different list; right-click again and confirm
//!    the menu updated (lazy global read).

use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use objc2::declare_class;
use objc2::mutability;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, NSObject, Sel};
use objc2::{msg_send, msg_send_id, sel, ClassType, DeclaredClass};
use objc2_app_kit::{
    NSApplication, NSControlStateValueOff, NSControlStateValueOn, NSEventModifierFlags, NSMenu,
    NSMenuItem,
};
use objc2_foundation::{MainThreadMarker, NSString};
use tokio::sync::mpsc::UnboundedSender;

use super::{Accelerator, MenuItemKind, MenuItemSpec};
use crate::{DockError, MenuClickEvent};

/// The currently-installed dock `NSMenu`, retained (+1). Main-thread-only:
/// written by `setMenu` (deno_core main thread) and read by
/// `applicationDockMenu:` (AppKit main thread), so the plain pointer swap is
/// sound. The global owns the +1; the IMP returns it +0 (borrowed) to AppKit.
static DOCK_MENU: AtomicPtr<NSMenu> = AtomicPtr::new(std::ptr::null_mut());

/// The shared click-handler target, retained for the app's lifetime. AppKit
/// holds menu-item targets weakly, so it must be kept alive here.
static TARGET: AtomicPtr<DockMenuTarget> = AtomicPtr::new(std::ptr::null_mut());

/// Maps a menu item's integer `tag` to its app-supplied `id`, rebuilt on every
/// `setMenu`. Indexed by tag; empty string means "no id" (no event emitted).
static ITEM_IDS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Sender for menu-click events; set once during state init.
static CLICK_TX: OnceLock<UnboundedSender<MenuClickEvent>> = OnceLock::new();

/// Whether `applicationDockMenu:` has been injected into the delegate's class.
static DELEGATE_PATCHED: AtomicBool = AtomicBool::new(false);

/// Register the channel sender the click handler pushes events through.
pub fn set_click_sender(tx: UnboundedSender<MenuClickEvent>) {
    let _ = CLICK_TX.set(tx);
}

// ============================================================================
// Click-handler target object
// ============================================================================

declare_class!(
    /// An Obj-C object that receives `dockMenuItemClicked:` actions from menu
    /// items and forwards the clicked item's id to the Deno side.
    struct DockMenuTarget;

    unsafe impl ClassType for DockMenuTarget {
        type Super = NSObject;
        type Mutability = mutability::Immutable;
        const NAME: &'static str = "ForgeDockMenuTarget";
    }

    impl DeclaredClass for DockMenuTarget {}

    unsafe impl DockMenuTarget {
        #[method(dockMenuItemClicked:)]
        fn dock_menu_item_clicked(&self, sender: &NSMenuItem) {
            let tag = unsafe { sender.tag() };
            let id = ITEM_IDS
                .lock()
                .ok()
                .and_then(|ids| ids.get(tag as usize).cloned());
            if let Some(id) = id {
                if !id.is_empty() {
                    if let Some(tx) = CLICK_TX.get() {
                        let _ = tx.send(MenuClickEvent {
                            id,
                            timestamp_ms: now_millis(),
                        });
                    }
                }
            }
        }
    }
);

impl DockMenuTarget {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}

/// Get (creating once) the shared click-handler target.
fn shared_target(mtm: MainThreadMarker) -> &'static DockMenuTarget {
    let existing = TARGET.load(Ordering::Acquire);
    if !existing.is_null() {
        // SAFETY: once stored, the target lives for the app's lifetime.
        return unsafe { &*existing };
    }
    let target = DockMenuTarget::new(mtm);
    let ptr = Retained::into_raw(target);
    // Store if still unset; if another init raced us, free ours and use theirs.
    match TARGET.compare_exchange(
        std::ptr::null_mut(),
        ptr,
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => unsafe { &*ptr },
        Err(winner) => {
            // SAFETY: reclaim the retain we just leaked on the losing pointer.
            unsafe { drop(Retained::from_raw(ptr)) };
            unsafe { &*winner }
        }
    }
}

// ============================================================================
// Menu construction
// ============================================================================

/// Build, install, and wire the dock menu from a validated spec tree.
///
/// Must run on the main thread. Returns an error if the app delegate is not yet
/// available to receive the injected `applicationDockMenu:` method (the app
/// should call `setMenu` after creating its first window).
pub fn install_menu(specs: &[MenuItemSpec]) -> Result<(), DockError> {
    let Some(mtm) = MainThreadMarker::new() else {
        return Err(DockError::menu_error(
            "dock.setMenu must be called on the main thread",
        ));
    };

    let target = shared_target(mtm);

    // Build the menu and the tag->id table together so each item's tag indexes
    // into the table the click handler reads.
    let mut id_table: Vec<String> = Vec::new();
    let menu = build_menu(specs, target, &mut id_table, mtm);

    *ITEM_IDS.lock().expect("ITEM_IDS poisoned") = id_table;
    store_menu(menu);
    inject_dock_menu_method(mtm)
}

/// Recursively build an `NSMenu` from specs, assigning each actionable item a
/// tag that indexes `id_table`.
fn build_menu(
    specs: &[MenuItemSpec],
    target: &DockMenuTarget,
    id_table: &mut Vec<String>,
    mtm: MainThreadMarker,
) -> Retained<NSMenu> {
    let menu = NSMenu::new(mtm);
    // Dock menus manage their own enabled-state; disable auto-enabling so our
    // explicit setEnabled flags are honored.
    unsafe { menu.setAutoenablesItems(false) };
    for spec in specs {
        let item = build_item(spec, target, id_table, mtm);
        menu.addItem(&item);
    }
    menu
}

fn build_item(
    spec: &MenuItemSpec,
    target: &DockMenuTarget,
    id_table: &mut Vec<String>,
    mtm: MainThreadMarker,
) -> Retained<NSMenuItem> {
    if spec.kind == MenuItemKind::Separator {
        return NSMenuItem::separatorItem(mtm);
    }

    let title = NSString::from_str(&spec.label);
    let key = spec
        .accelerator
        .as_ref()
        .map(|a| a.key.as_str())
        .unwrap_or("");
    let key_equiv = NSString::from_str(key);
    let action = Some(sel!(dockMenuItemClicked:));

    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(mtm.alloc(), &title, action, &key_equiv)
    };

    unsafe {
        item.setTarget(Some(target.as_ref()));
        item.setEnabled(spec.enabled);

        if spec.kind == MenuItemKind::Checkbox {
            item.setState(if spec.checked {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }

        if let Some(acc) = &spec.accelerator {
            item.setKeyEquivalentModifierMask(modifier_mask(acc));
        }

        let tag = id_table.len() as isize;
        item.setTag(tag);
        id_table.push(spec.id.clone().unwrap_or_default());

        if !spec.submenu.is_empty() {
            let submenu = build_menu(&spec.submenu, target, id_table, mtm);
            item.setSubmenu(Some(&submenu));
        }
    }

    item
}

/// Translate parsed accelerator modifiers into `NSEventModifierFlags`.
fn modifier_mask(acc: &Accelerator) -> NSEventModifierFlags {
    let mut mask = NSEventModifierFlags(0);
    if acc.modifiers.command {
        mask |= NSEventModifierFlags::NSEventModifierFlagCommand;
    }
    if acc.modifiers.control {
        mask |= NSEventModifierFlags::NSEventModifierFlagControl;
    }
    if acc.modifiers.option {
        mask |= NSEventModifierFlags::NSEventModifierFlagOption;
    }
    if acc.modifiers.shift {
        mask |= NSEventModifierFlags::NSEventModifierFlagShift;
    }
    mask
}

/// Store `menu` as the active dock menu (+1), releasing the previously-stored
/// menu. Main-thread-only.
fn store_menu(menu: Retained<NSMenu>) {
    let new_ptr = Retained::into_raw(menu);
    let old = DOCK_MENU.swap(new_ptr, Ordering::AcqRel);
    if !old.is_null() {
        // SAFETY: the global owned the previous +1; reclaim and release it.
        unsafe { drop(Retained::from_raw(old)) };
    }
}

// ============================================================================
// Delegate injection
// ============================================================================

/// The IMP backing `applicationDockMenu:`. Reads the current menu lazily from
/// the global and returns it +0 (AppKit does not take ownership; the global
/// keeps the +1 alive).
extern "C" fn application_dock_menu(
    _this: *mut AnyObject,
    _cmd: Sel,
    _sender: *mut AnyObject,
) -> *mut NSMenu {
    DOCK_MENU.load(Ordering::Acquire)
}

/// Add `applicationDockMenu:` to the app delegate's class, once. The method is
/// installed on the delegate's existing class so tao's lifecycle handling is
/// untouched.
fn inject_dock_menu_method(mtm: MainThreadMarker) -> Result<(), DockError> {
    if DELEGATE_PATCHED.load(Ordering::Acquire) {
        return Ok(());
    }

    let app = NSApplication::sharedApplication(mtm);
    let Some(delegate) = (unsafe { app.delegate() }) else {
        return Err(DockError::menu_error(
            "no application delegate yet; call dock.setMenu after creating a window",
        ));
    };

    // The delegate is a ProtocolObject; ask it for its concrete runtime class.
    let cls: &AnyClass = unsafe { msg_send![&*delegate, class] };
    let sel = sel!(applicationDockMenu:);

    // If the delegate already provides the method, leave it; otherwise add ours.
    if cls.instance_method(sel).is_none() {
        // Type encoding: returns id (@), args self (@), _cmd (:), sender (@).
        let types = c"@@:@";
        let imp: objc2::ffi::IMP = Some(unsafe {
            std::mem::transmute::<*const (), unsafe extern "C" fn()>(
                application_dock_menu as *const (),
            )
        });
        let added = unsafe {
            objc2::ffi::class_addMethod(
                cls as *const _ as *mut objc2::ffi::objc_class,
                sel.as_ptr(),
                imp,
                types.as_ptr(),
            )
        };
        if added == objc2::ffi::NO {
            return Err(DockError::menu_error(
                "failed to install applicationDockMenu: on the app delegate",
            ));
        }

        // AppKit snapshots the delegate's `respondsToSelector:` set at
        // `setDelegate:` time. tao installed this delegate *before* we added the
        // method, so without forcing a re-resolve AppKit would never call our
        // freshly-injected `applicationDockMenu:`. Re-set the delegate to bust
        // that cache. Some AppKit versions early-out when the pointer is
        // unchanged, so clear to nil first, then restore. We hold a +1 on
        // `delegate`, so it stays alive across the swap.
        app.setDelegate(None);
        app.setDelegate(Some(&delegate));
    }

    DELEGATE_PATCHED.store(true, Ordering::Release);
    Ok(())
}

/// Current time in epoch milliseconds as an `f64`.
fn now_millis() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}
