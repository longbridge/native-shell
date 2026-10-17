//! Canvas. (Being ported: see `nsh-platform-windows/src/canvas.rs`.)

use std::{rc::Rc, sync::Arc};

use nsh_animation::AnimationValue;
use nsh_core::canvas::Drawing;

use crate::{
    render::DrawCx,
    surface::Surface,
    view::{NshView, ViewState},
};

/// A canvas view's drawing.
pub(crate) struct CanvasState;

/// A canvas view was made.
pub(crate) fn create(_view: &NshView) {}

/// `Canvas`: a new drawing.
pub(crate) fn set(_surface: &Rc<Surface>, _view: &NshView, _drawing: Arc<Drawing>) {}

/// An animated slot of the drawing changed.
pub(crate) fn set_arg(_view: &NshView, _k: u16, _value: AnimationValue) {}

/// Draws a canvas view's drawing.
pub(crate) fn draw(_cx: &mut DrawCx<'_>, _state: &ViewState) {}
