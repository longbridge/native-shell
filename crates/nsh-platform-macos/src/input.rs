//! Pointer and keyboard input: the root view's mouse, wheel and trackpad
//! events, its own hit testing over the views, touches for the responder
//! system, presses, hover and native gestures.

use std::rc::Rc;

use nsh_core::{Command, GestureValues, NodeId};
use objc2_app_kit::NSEvent;

use crate::{surface::Surface, view::NshView};

/// Pointer input state of a surface.
#[derive(Default)]
pub(crate) struct Input {}

/// A mouse, wheel or trackpad event over the root view.
pub(crate) fn event(_surface: &Rc<Surface>, _event: &NSEvent) {}

/// A key went down (`down`) or up; true if it was handled.
pub(crate) fn key(_surface: &Rc<Surface>, _event: &NSEvent, _down: bool) -> bool {
    false
}

/// Forgets everything (the app stopped).
pub(crate) fn reset(_surface: &Surface) {}

/// Which interaction states of a view JS tracks.
pub(crate) fn track_states(_surface: &Rc<Surface>, view: &NshView, states: u8) {
    view.state_mut().tracked = states;
}

/// A view's gesture values (pan, pinch, rotation).
pub(crate) fn gestures(_surface: &Rc<Surface>, _view: &NshView, _values: GestureValues) {}

/// A view's events changed.
pub(crate) fn events_changed(_surface: &Rc<Surface>, _view: &NshView) {}

/// A view's props changed.
pub(crate) fn props_changed(_surface: &Rc<Surface>, _view: &NshView) {}

/// A view's style changed (cursor).
pub(crate) fn style_changed(_view: &NshView) {}

/// The JS responder commands.
pub(crate) fn command(_surface: &Rc<Surface>, _id: NodeId, _command: &Command) {}
