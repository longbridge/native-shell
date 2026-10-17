//! Window chrome: title, menus. (Being ported: see
//! `nsh-platform-windows/src/chrome.rs`.)

use std::rc::Rc;

use crate::{input::KeyEvent, surface::Surface};

/// The window's chrome state.
#[derive(Default)]
pub(crate) struct Chrome;

/// An application menu shortcut. True when it took the key.
pub(crate) fn shortcut(_surface: &Rc<Surface>, _event: &KeyEvent) -> bool {
    false
}

/// What an item of a menu of the platform's own does.
pub(crate) type Action = Rc<dyn Fn(&Rc<Surface>)>;

/// An item of a menu of the platform's own ([`popup_actions`]).
#[allow(dead_code)] // Until the menus are ported.
pub(crate) enum LocalItem {
    /// A line between groups.
    Separator,
    /// An item.
    Item {
        /// Its role: the label and shortcut shown (the action does the
        /// work).
        role: Option<nsh_core::desktop::MenuRole>,
        /// A label of its own (else the role's).
        label: &'static str,
        /// It can be chosen.
        enabled: bool,
        /// What choosing it does.
        action: Action,
    },
}

/// Shows a context menu of the platform's own (a text field's) at the
/// pointer.
pub(crate) fn popup_actions(_surface: &Surface, items: Vec<LocalItem>) {
    log::debug!(
        "chrome: a context menu of {} items is not shown yet",
        items.len()
    );
}
