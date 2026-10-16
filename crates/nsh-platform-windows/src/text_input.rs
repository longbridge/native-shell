//! Text inputs: an editor drawn with DirectWrite (single and multiple
//! lines, selection, the clipboard, undo), with IME composition through
//! IMM32.

use std::rc::Rc;

use nsh_core::{InputConfig, Rect};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

use crate::{surface::Surface, view::NshView};

/// An input view's editor.
pub(crate) struct InputState {}

/// Sets up an input view.
pub(crate) fn create(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// `Input`.
pub(crate) fn config(surface: &Rc<Surface>, view: &NshView, config: &InputConfig) {
    let _ = (surface, view, config);
}

/// `InputValue`.
pub(crate) fn value(view: &NshView, text: &str, count: u32) {
    let _ = (view, text, count);
}

/// The view's animated values changed (text color).
pub(crate) fn animated(view: &NshView) {
    let _ = view;
}

/// `InputContent`: where the text goes inside the view.
pub(crate) fn content(view: &NshView, content: Rect) {
    let _ = (view, content);
}

/// An input accessory view (never shown on desktops).
pub(crate) fn accessory(surface: &Rc<Surface>, parent: &NshView, child: &NshView) {
    let _ = (surface, parent, child);
}

/// `focus()`.
pub(crate) fn focus(view: &NshView) {
    let _ = view;
}

/// `blur()`.
pub(crate) fn blur(view: &NshView) {
    let _ = view;
}

/// Blurs whatever field has the focus.
pub(crate) fn blur_all(surface: &Rc<Surface>) {
    let _ = surface;
}

/// `setSelection`.
pub(crate) fn set_selection(view: &NshView, start: u32, end: u32) {
    let _ = (view, start, end);
}

/// Characters typed (`WM_CHAR`).
pub(crate) fn char_input(surface: &Rc<Surface>, text: &str) {
    let _ = (surface, text);
}

/// IME messages; `None` for default handling.
pub(crate) fn ime(
    surface: &Rc<Surface>,
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    let _ = (surface, hwnd, msg, wparam, lparam);
    None
}

/// The window got (or lost) the keyboard focus.
pub(crate) fn window_focus(surface: &Rc<Surface>, focused: bool) {
    let _ = (surface, focused);
}

/// A pointer event on input `view` (its coordinates); true when handled.
pub(crate) fn pointer(surface: &Rc<Surface>, view: &NshView, event: &crate::view::ItemPointer) -> bool {
    let _ = (surface, view, event);
    false
}

/// A key for the focused field; true when handled.
pub(crate) fn key(surface: &Rc<Surface>, event: &crate::input::KeyEvent) -> bool {
    let _ = (surface, event);
    false
}

/// The field that has the keyboard focus.
pub(crate) fn focused(surface: &Surface) -> Option<NshView> {
    let _ = surface;
    None
}
