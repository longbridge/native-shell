//! The app's keyboard shortcuts (`nsh/keymap`): the table the engine
//! mounts ([`MountItem::Keymap`](nsh_core::MountItem::Keymap)) becomes a
//! global-scope `gtk::ShortcutController` on the root, one shortcut per
//! binding, newest first (GTK tries a controller's shortcuts in order).
//!
//! GTK matches the triggers, also on the keys of the layout's Latin group
//! under other layouts, as it does the application's accelerators. A
//! binding's action decides on the spot whether it fires (where the focus
//! is: `KeyBinding::in_text`), reports it to JS with the root's
//! `KeyCommand` event and stops the key; JS runs the handler afterwards.
//! A binding that does not fire lets GTK try the next shortcut and then the
//! focus widget.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

use gtk4::{self as gtk, glib, prelude::*};
use nsh_core::{
    EventKind, EventPayload, PayloadValue, ROOT_ID,
    keymap::{Key, Keymap, Keystroke},
    platform::Os,
};

use crate::surface::Surface;

/// A surface's shortcut controller, while it has bindings.
#[derive(Default)]
pub(crate) struct KeymapState {
    controller: RefCell<Option<gtk::ShortcutController>>,
    /// The binding whose key is held down (GDK does not mark auto-repeats):
    /// it does not fire again until a key is released.
    held: Rc<Cell<Option<u32>>>,
    /// The window's key controller that sees releases, and its window.
    releases: RefCell<Option<(glib::WeakRef<gtk::Window>, gtk::EventControllerKey)>>,
}

/// `MountItem::Keymap`: replaces the surface's shortcuts.
pub(crate) fn set(surface: &Rc<Surface>, keymap: Arc<Keymap>) {
    clear(surface);
    if keymap.is_empty() {
        return;
    }
    let controller = gtk::ShortcutController::new();
    controller.set_scope(gtk::ShortcutScope::Global);
    for binding in keymap.bindings().iter().rev() {
        // The dev host's Ctrl+R reloads, whatever the app binds.
        if cfg!(feature = "dev") && binding.keystroke.reloads(Os::Linux) {
            continue;
        }
        let Some(trigger) = trigger(&binding.keystroke) else {
            continue;
        };
        let weak = Rc::downgrade(surface);
        let (id, in_text, repeat) = (binding.id, binding.in_text, binding.repeat);
        let action = gtk::CallbackAction::new(move |_, _| match weak.upgrade() {
            Some(surface) if fires(&surface, in_text) => {
                watch_releases(&surface);
                let held = &surface.keymap.held;
                if repeat || held.get() != Some(id) {
                    held.set(Some(id));
                    surface.event(
                        ROOT_ID,
                        EventKind::KeyCommand,
                        EventPayload::Value(PayloadValue::Number(id as f64)),
                    );
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        });
        let shortcut =
            gtk::Shortcut::new(gtk::ShortcutTrigger::parse_string(&trigger), Some(action));
        controller.add_shortcut(shortcut);
    }
    surface.root.add_controller(controller.clone());
    *surface.keymap.controller.borrow_mut() = Some(controller);
}

/// Removes the surface's shortcuts (the app stopped).
pub(crate) fn clear(surface: &Surface) {
    if let Some(controller) = surface.keymap.controller.borrow_mut().take() {
        surface.root.remove_controller(&controller);
    }
    if let Some((window, keys)) = surface.keymap.releases.borrow_mut().take()
        && let Some(window) = window.upgrade()
    {
        window.remove_controller(&keys);
    }
    surface.keymap.held.set(None);
}

/// Listens to the releases of the surface's window (once per window), so
/// that a held key's binding fires again after it is released.
fn watch_releases(surface: &Surface) {
    let Some(window) = surface.root.root().and_downcast::<gtk::Window>() else {
        return;
    };
    let mut releases = surface.keymap.releases.borrow_mut();
    if releases
        .as_ref()
        .is_some_and(|(w, _)| w.upgrade().as_ref() == Some(&window))
    {
        return;
    }
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let held = surface.keymap.held.clone();
    keys.connect_key_released(move |_, _, _, _| held.set(None));
    window.add_controller(keys.clone());
    *releases = Some((window.downgrade(), keys));
}

/// Whether a binding fires where the focus is: never while it is in
/// another surface, in a text field or hosted widget only when the binding
/// may (`in_text`).
fn fires(surface: &Surface, in_text: bool) -> bool {
    let root = surface.root.upcast_ref::<gtk::Widget>();
    let Some(focus) = root.root().and_then(|r| r.focus()) else {
        return true;
    };
    if !(focus.is_ancestor(root) || &focus == root) {
        return false;
    }
    // A field's text widget, or a hosted widget (a web view).
    let editing = !focus.is::<crate::view::NshView>();
    in_text || !editing
}

/// The GTK trigger of `keystroke` (`<Control><Shift>p`).
fn trigger(keystroke: &Keystroke) -> Option<String> {
    let m = keystroke.modifiers;
    let mut out = String::new();
    for (on, name) in [
        (m.ctrl, "<Control>"),
        (m.alt, "<Alt>"),
        (m.shift, "<Shift>"),
        (m.cmd, "<Super>"),
    ] {
        if on {
            out.push_str(name);
        }
    }
    let name = match keystroke.key {
        Key::Char(c) if c.is_ascii_alphanumeric() => c.to_string(),
        Key::Char(c) => match c {
            ' ' => "space",
            '-' => "minus",
            '=' => "equal",
            '[' => "bracketleft",
            ']' => "bracketright",
            '\\' => "backslash",
            ';' => "semicolon",
            '\'' => "apostrophe",
            ',' => "comma",
            '.' => "period",
            '/' => "slash",
            '`' => "grave",
            _ => return None,
        }
        .to_owned(),
        Key::Enter => "Return".to_owned(),
        Key::Escape => "Escape".to_owned(),
        Key::Tab => "Tab".to_owned(),
        Key::Backspace => "BackSpace".to_owned(),
        Key::Delete => "Delete".to_owned(),
        Key::Up => "Up".to_owned(),
        Key::Down => "Down".to_owned(),
        Key::Left => "Left".to_owned(),
        Key::Right => "Right".to_owned(),
        Key::Home => "Home".to_owned(),
        Key::End => "End".to_owned(),
        Key::PageUp => "Page_Up".to_owned(),
        Key::PageDown => "Page_Down".to_owned(),
        Key::F(n) => format!("F{n}"),
    };
    out.push_str(&name);
    Some(out)
}

#[cfg(test)]
mod tests {
    use nsh_core::{keymap::Keystroke, platform::Os};

    #[test]
    fn triggers() {
        let t = |s| super::trigger(&Keystroke::parse(s, Os::Linux).unwrap()).unwrap();
        assert_eq!(t("mod+shift+p"), "<Control><Shift>p");
        assert_eq!(t("alt+enter"), "<Alt>Return");
        assert_eq!(t("cmd+["), "<Super>bracketleft");
        assert_eq!(t("f5"), "F5");
    }
}
