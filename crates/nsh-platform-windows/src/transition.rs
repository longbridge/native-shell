//! Layout animations, style transitions and keyframes.

use std::rc::Rc;

use nsh_core::{LayoutAnimation, Rect, ViewStyle};

use crate::{geom::Mat4, surface::Surface, view::NshView};

/// The running transitions of a view.
pub(crate) struct Live {
    /// Extra opacity (fades).
    pub(crate) opacity: f32,
}

impl Live {
    /// The extra transform of running transitions.
    pub(crate) fn extra(&self) -> Option<Extra> {
        None
    }
}

/// An extra transform of running transitions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Extra {}

impl Extra {
    /// Applies it to `m` (a view `height` points tall).
    pub(crate) fn apply(self, m: Mat4, height: f32) -> Mat4 {
        let _ = height;
        m
    }
}

/// A view was inserted.
pub(crate) fn inserted(surface: &Rc<Surface>, view: &NshView, animation: Option<&LayoutAnimation>) {
    let _ = (surface, view, animation);
}

/// A view is removed: true when an animation detaches it later.
pub(crate) fn removing(
    surface: &Rc<Surface>,
    view: &NshView,
    animation: Option<&LayoutAnimation>,
) -> bool {
    let _ = (surface, view, animation);
    false
}

/// Where a running layout animation takes the view.
pub(crate) fn frame_target(view: &NshView) -> Option<Rect> {
    let _ = view;
    None
}

/// The view's frame changed from `old`.
pub(crate) fn frame_changed(
    surface: &Rc<Surface>,
    view: &NshView,
    old: Rect,
    animation: Option<&LayoutAnimation>,
) {
    let _ = (surface, view, old, animation);
}

/// The view's style changed from `old`.
pub(crate) fn style_changed(surface: &Rc<Surface>, view: &NshView, old: &ViewStyle) {
    let _ = (surface, view, old);
}
