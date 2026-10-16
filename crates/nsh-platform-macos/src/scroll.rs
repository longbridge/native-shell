//! Scroll views: offsets of their own, moved by wheels, trackpads, drags
//! and the keyboard, shown by their container's bounds origin.

use std::rc::Rc;

use nsh_core::{Command, ScrollBehavior, Size, StickyCell};

use crate::{surface::Surface, view::NshView};

/// Gestures, glides and the indicator of one scroll view.
#[derive(Debug, Default, Clone)]
pub(crate) struct Live {}

/// A child was inserted.
pub(crate) fn child_inserted(_view: &NshView) {}

/// Direction, indicator and mirroring.
pub(crate) fn config(
    _surface: &Rc<Surface>,
    view: &NshView,
    horizontal: bool,
    shows_indicator: bool,
    mirrored: bool,
) {
    if let Some(s) = view.state_mut().scroll.as_mut() {
        s.horizontal = horizontal;
        s.indicator = shows_indicator;
        s.mirrored = mirrored;
    }
}

/// The content's size.
pub(crate) fn content(_surface: &Rc<Surface>, view: &NshView, size: Size) {
    if let Some(s) = view.state_mut().scroll.as_mut() {
        s.content = (size.width, size.height);
    }
}

/// The core scrolls by `(dx, dy)`.
pub(crate) fn scroll_by(_surface: &Rc<Surface>, _view: &NshView, _dx: f32, _dy: f32) {}

/// Sticky cells.
pub(crate) fn sticky(_surface: &Rc<Surface>, view: &NshView, cells: Vec<StickyCell>) {
    if let Some(s) = view.state_mut().scroll.as_mut() {
        s.sticky = cells;
    }
}

/// Paging, snapping, insets...
pub(crate) fn behavior(_surface: &Rc<Surface>, view: &NshView, behavior: &ScrollBehavior) {
    if let Some(s) = view.state_mut().scroll.as_mut() {
        s.behavior = behavior.clone();
    }
}

/// Pull to refresh.
pub(crate) fn refreshing(_surface: &Rc<Surface>, _view: &NshView, _refreshing: bool) {}

/// `scrollTo`, `scrollToEnd`, `scrollToIndex`, `zoomTo`.
pub(crate) fn command(_surface: &Rc<Surface>, _view: &NshView, _command: &Command) {}
