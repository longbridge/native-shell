//! `<Canvas>`: the recorded drawing replayed with Core Graphics in
//! `drawRect:`.

use std::{rc::Rc, sync::Arc};

use nsh_animation::AnimationValue;
use nsh_core::canvas::Drawing;
use objc2_core_graphics::CGContext;

use crate::{
    surface::Surface,
    view::{NshView, ViewState},
};

/// A canvas view's drawing.
#[derive(Default)]
pub(crate) struct CanvasState {
    drawing: Option<Arc<Drawing>>,
}

/// Sets up a canvas view.
pub(crate) fn create(view: &NshView) {
    view.state_mut().canvas = Some(CanvasState::default());
}

/// A canvas got a new drawing.
pub(crate) fn set(_surface: &Rc<Surface>, view: &NshView, drawing: Arc<Drawing>) {
    if let Some(c) = view.state_mut().canvas.as_mut() {
        c.drawing = Some(drawing);
    }
    view.changed();
}

/// An animated slot value of the drawing changed.
pub(crate) fn set_arg(_view: &NshView, _k: u16, _value: AnimationValue) {}

/// Draws a canvas view's drawing (y-down context).
pub(crate) fn draw(_ctx: &CGContext, _state: &ViewState) {}
