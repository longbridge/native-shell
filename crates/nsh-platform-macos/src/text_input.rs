//! `<Input>`: `NSTextField` / `NSTextView` hosted at the content box.

use std::rc::Rc;

use nsh_core::{InputConfig, Rect};

use crate::{surface::Surface, view::NshView};

/// A text input's native field.
pub(crate) struct InputState {}

/// Sets up an input view.
pub(crate) fn create(_surface: &Rc<Surface>, _view: &NshView) {}

/// The input's configuration.
pub(crate) fn config(_surface: &Rc<Surface>, _view: &NshView, _config: &InputConfig) {}

/// JS set the value (`count`: JS's edit count).
pub(crate) fn value(_view: &NshView, _text: &str, _count: u32) {}

/// An animated value (text color) changed.
pub(crate) fn animated(_view: &NshView) {}

/// The content box.
pub(crate) fn content(_view: &NshView, _content: Rect) {}

/// An input accessory view (never shown on desktops).
pub(crate) fn accessory(_surface: &Rc<Surface>, _parent: &NshView, _child: &NshView) {}

/// `focus()`.
pub(crate) fn focus(_view: &NshView) {}

/// `blur()`.
pub(crate) fn blur(_view: &NshView) {}

/// Blurs every input of the surface.
pub(crate) fn blur_all(_surface: &Rc<Surface>) {}

/// `setSelection`.
pub(crate) fn set_selection(_view: &NshView, _start: u32, _end: u32) {}
