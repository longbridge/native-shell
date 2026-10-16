//! Layout animations (`LayoutAnimation`), style transitions and keyframes,
//! run on the display link.

use std::rc::Rc;

use nsh_core::{LayoutAnimation, Matrix, Rect, ViewStyle};

use crate::{surface::Surface, view::NshView};

/// What animates on one view, and the values placing it reads.
pub(crate) struct Live {
    /// Opacity multiplier (appearing / disappearing, keyframes).
    pub(crate) opacity: f32,
}

impl Live {
    /// The transform to apply before the style's, if any.
    pub(crate) fn extra(&self) -> Option<Extra> {
        None
    }
}

/// A rotation, scale and vertical shift (a fraction of the height)
/// around the transform origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Extra {}

impl Extra {
    /// `E · m`: this transform, then `m` (row vectors, in the view's local
    /// space about its transform origin).
    pub(crate) fn apply(self, m: Matrix, _height: f32) -> Matrix {
        m
    }
}

/// A view was inserted (it may animate in).
pub(crate) fn inserted(
    _surface: &Rc<Surface>,
    _view: &NshView,
    _animation: Option<&LayoutAnimation>,
) {
}

/// A view is being removed: true if it animates out (and detaches itself
/// when done).
pub(crate) fn removing(
    _surface: &Rc<Surface>,
    _view: &NshView,
    _animation: Option<&LayoutAnimation>,
) -> bool {
    false
}

/// Where a running layout animation moves the view.
pub(crate) fn frame_target(_view: &NshView) -> Option<Rect> {
    None
}

/// A view's frame changed from `old`.
pub(crate) fn frame_changed(
    _surface: &Rc<Surface>,
    _view: &NshView,
    _old: Rect,
    _animation: Option<&LayoutAnimation>,
) {
}

/// A view's style changed from `old` (transitions, keyframes).
pub(crate) fn style_changed(_surface: &Rc<Surface>, _view: &NshView, _old: &ViewStyle) {}
