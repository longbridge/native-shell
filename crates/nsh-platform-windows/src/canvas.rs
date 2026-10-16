//! Canvas: the recorded drawing replayed with Direct2D.

use std::{rc::Rc, sync::Arc};

use nsh_animation::AnimationValue;
use nsh_core::canvas::Drawing;

use crate::{
    render::DrawCx,
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

/// `Canvas`.
pub(crate) fn set(surface: &Rc<Surface>, view: &NshView, drawing: Arc<Drawing>) {
    let _ = surface;
    if let Some(c) = view.state_mut().canvas.as_mut() {
        c.drawing = Some(drawing);
    }
    view.changed();
}

/// A native value bound to slot `k` changed.
pub(crate) fn set_arg(view: &NshView, k: u16, value: AnimationValue) {
    let _ = (view, k, value);
}

/// Draws the canvas.
pub(crate) fn draw(cx: &mut DrawCx<'_>, state: &ViewState) {
    let _ = (cx, state);
}
