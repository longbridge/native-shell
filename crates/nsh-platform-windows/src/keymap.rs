//! The app's keyboard shortcuts (`nsh/keymap`): the table the engine
//! mounts ([`MountItem::Keymap`](nsh_core::MountItem::Keymap)), matched on
//! every key down the window gets ([`crate::input::key`]): after the dialogs
//! the surface shows, before the focused field, selectable text, navigation
//! and scroll views. A key that fires a binding is consumed (its `WM_CHAR`
//! too) and reported to JS with the root's `KeyCommand` event; JS runs the
//! handler afterwards.
//!
//! Keys are matched by virtual key code: letters and digits are the same
//! on every layout (a Cyrillic layout's keys have the US layout's codes),
//! punctuation by the US layout's `VK_OEM_*` keys.

use std::{cell::RefCell, rc::Rc, sync::Arc};

use nsh_core::{
    EventKind, EventPayload, PayloadValue, ROOT_ID,
    keymap::{Key, KeyMatch, Keymap, Keystroke, Modifiers},
    platform::Os,
};
use windows::Win32::UI::WindowsAndMessaging::{MSG, PM_REMOVE, PeekMessageW, WM_CHAR, WM_SYSCHAR};

use crate::{input::KeyEvent, surface::Surface};

/// A surface's table, while it has bindings.
#[derive(Default)]
pub(crate) struct KeymapState {
    table: RefCell<Option<Arc<Keymap>>>,
}

/// `MountItem::Keymap`: replaces the surface's table.
pub(crate) fn set(surface: &Surface, keymap: Arc<Keymap>) {
    *surface.keymap.table.borrow_mut() = (!keymap.is_empty()).then_some(keymap);
}

/// Drops the table (the app stopped).
pub(crate) fn clear(surface: &Surface) {
    surface.keymap.table.borrow_mut().take();
}

/// A key down: whether it fired (or, held, repeats) one of the bindings;
/// then it is consumed.
pub(crate) fn key(surface: &Rc<Surface>, event: &KeyEvent) -> bool {
    let Some(table) = surface.keymap.table.borrow().clone() else {
        return false;
    };
    let Some(keystroke) = keystroke(event) else {
        return false;
    };
    // The dev host's Ctrl+R reloads, whatever the app binds.
    if cfg!(feature = "dev") && keystroke.reloads(Os::Windows) {
        return false;
    }
    let editing = crate::text_input::focused(surface).is_some();
    let found = table.find(keystroke, editing, event.repeat);
    if let KeyMatch::Fire(id) = found {
        surface.event(
            ROOT_ID,
            EventKind::KeyCommand,
            EventPayload::Value(PayloadValue::Number(id as f64)),
        );
    }
    if found.consumed() {
        drop_char(surface);
    }
    found.consumed()
}

/// Removes the character `TranslateMessage` queued for the consumed key.
fn drop_char(surface: &Surface) {
    let mut msg = MSG::default();
    for kind in [WM_CHAR, WM_SYSCHAR] {
        // SAFETY: a plain call with a local out value, for our own window.
        unsafe {
            let _ = PeekMessageW(&mut msg, Some(surface.hwnd()), kind, kind, PM_REMOVE);
        }
    }
}

/// The keystroke of a key down.
fn keystroke(event: &KeyEvent) -> Option<Keystroke> {
    let m = event.modifiers;
    let modifiers = Modifiers {
        ctrl: m.ctrl,
        alt: m.alt,
        shift: m.shift,
        cmd: m.meta,
    };
    Some(Keystroke {
        modifiers,
        key: key_of(event.vk)?,
    })
}

/// The key of virtual key code `vk` (`VK_*`).
fn key_of(vk: u16) -> Option<Key> {
    Some(match vk {
        0x08 => Key::Backspace,
        0x09 => Key::Tab,
        0x0D => Key::Enter,
        0x1B => Key::Escape,
        0x20 => Key::Char(' '),
        0x21 => Key::PageUp,
        0x22 => Key::PageDown,
        0x23 => Key::End,
        0x24 => Key::Home,
        0x25 => Key::Left,
        0x26 => Key::Up,
        0x27 => Key::Right,
        0x28 => Key::Down,
        0x2D => Key::Insert,
        0x2E => Key::Delete,
        // `0`-`9`, `A`-`Z`.
        0x30..=0x39 => Key::Char(char::from(vk as u8)),
        0x41..=0x5A => Key::Char(char::from(vk as u8).to_ascii_lowercase()),
        // The keypad's digits.
        0x60..=0x69 => Key::Char(char::from(b'0' + (vk - 0x60) as u8)),
        // F1 to F24.
        0x70..=0x87 => Key::F((vk - 0x6F) as u8),
        // `VK_OEM_*`: the US layout's punctuation keys.
        0xBA => Key::Char(';'),
        0xBB => Key::Char('='),
        0xBC => Key::Char(','),
        0xBD => Key::Char('-'),
        0xBE => Key::Char('.'),
        0xBF => Key::Char('/'),
        0xC0 => Key::Char('`'),
        0xDB => Key::Char('['),
        0xDC => Key::Char('\\'),
        0xDD => Key::Char(']'),
        0xDE => Key::Char('\''),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use nsh_core::keymap::Key;

    #[test]
    fn virtual_keys() {
        assert_eq!(super::key_of(0x4B), Some(Key::Char('k')));
        assert_eq!(super::key_of(0x31), Some(Key::Char('1')));
        assert_eq!(super::key_of(0x74), Some(Key::F(5)));
        assert_eq!(super::key_of(0xBF), Some(Key::Char('/')));
        assert_eq!(super::key_of(0x10), None);
    }
}
