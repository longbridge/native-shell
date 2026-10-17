//! Text inputs. (Being ported: see `nsh-platform-windows/src/text_input.rs`;
//! [`model`] is the editing model, shared as is.)

#[allow(dead_code)]
mod model;

use std::rc::Rc;

use nsh_core::{InputConfig, Rect};

use crate::{
    input::KeyEvent,
    surface::Surface,
    view::{ItemPointer, NshView},
};

/// A text input's editor.
pub(crate) struct InputState;

/// An input view was made.
pub(crate) fn create(_surface: &Rc<Surface>, _view: &NshView) {}
/// An input accessory view was inserted (never shown on the desktop).
pub(crate) fn accessory(_surface: &Rc<Surface>, _parent: &NshView, _child: &NshView) {}
/// An animated text color changed.
pub(crate) fn animated(_view: &NshView) {}
/// `Blur`.
pub(crate) fn blur(_view: &NshView) {}
/// Blurs the focused input.
pub(crate) fn blur_all(_surface: &Rc<Surface>) {}
/// `Input`.
pub(crate) fn config(_surface: &Rc<Surface>, _view: &NshView, _config: &InputConfig) {}
/// `InputContent`.
pub(crate) fn content(_view: &NshView, _content: Rect) {}
/// The input's context menu.
pub(crate) fn context_menu(_surface: &Rc<Surface>, _view: &NshView) {}
/// `Focus`.
pub(crate) fn focus(_view: &NshView) {}
/// The focused input.
pub(crate) fn focused(_surface: &Surface) -> Option<NshView> {
    None
}
/// A key for the focused input. True when taken.
pub(crate) fn key(_surface: &Rc<Surface>, _event: &KeyEvent) -> bool {
    false
}
/// A pointer event on an input. True when handled.
pub(crate) fn pointer(_surface: &Rc<Surface>, _view: &NshView, _event: &ItemPointer) -> bool {
    false
}
/// `SetSelection`.
pub(crate) fn set_selection(_view: &NshView, _start: u32, _end: u32) {}
/// Tab moved the focus to `view`.
pub(crate) fn tab_to(_view: &NshView) {}
/// `InputValue`.
pub(crate) fn value(_view: &NshView, _text: &str, _count: u32) {}
/// The text, access and kind of an input, for accessibility.
pub(crate) fn accessible_value(_view: &NshView) -> Option<crate::access::FieldValue> {
    None
}
/// An assistive technology sets an input's text. True when it was set.
pub(crate) fn set_accessible_value(_surface: &Rc<Surface>, _view: &NshView, _text: &str) -> bool {
    false
}
