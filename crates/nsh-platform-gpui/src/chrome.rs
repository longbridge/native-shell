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
