//! Scroll views: offsets, wheel and drag scrolling, the indicator.

use std::rc::Rc;

use nsh_core::{Command, ScrollBehavior, Size, StickyCell};

use crate::{
    render::DrawCx,
    surface::Surface,
    view::{NshView, ViewState},
};

/// Gestures, glides and the indicator of a scroll view.
#[derive(Default)]
pub(crate) struct Live {}

/// A child was inserted into a scroll view.
pub(crate) fn child_inserted(view: &NshView) {
    let _ = view;
}

/// `ScrollConfig`.
pub(crate) fn config(
    surface: &Rc<Surface>,
    view: &NshView,
    horizontal: bool,
    shows_indicator: bool,
    mirrored: bool,
) {
    let _ = surface;
    if let Some(s) = view.state_mut().scroll.as_mut() {
        s.horizontal = horizontal;
        s.indicator = shows_indicator;
        s.mirrored = mirrored;
    }
}

/// `ScrollContent`.
pub(crate) fn content(surface: &Rc<Surface>, view: &NshView, size: Size) {
    let _ = surface;
    if let Some(s) = view.state_mut().scroll.as_mut() {
        s.content = (size.width, size.height);
    }
}

/// `ScrollBy`.
pub(crate) fn scroll_by(surface: &Rc<Surface>, view: &NshView, dx: f32, dy: f32) {
    let _ = (surface, view, dx, dy);
}

/// `StickyCells`.
pub(crate) fn sticky(surface: &Rc<Surface>, view: &NshView, cells: Vec<StickyCell>) {
    let _ = surface;
    if let Some(s) = view.state_mut().scroll.as_mut() {
        s.sticky = cells;
    }
}

/// `ScrollBehavior`.
pub(crate) fn behavior(surface: &Rc<Surface>, view: &NshView, behavior: &ScrollBehavior) {
    let _ = surface;
    if let Some(s) = view.state_mut().scroll.as_mut() {
        s.behavior = behavior.clone();
    }
}

/// `Refreshing`.
pub(crate) fn refreshing(surface: &Rc<Surface>, view: &NshView, refreshing: bool) {
    let _ = (surface, view, refreshing);
}

/// `scrollTo`, `scrollToEnd`, `scrollToIndex`, `zoomTo`.
pub(crate) fn command(surface: &Rc<Surface>, view: &NshView, command: &Command) {
    let _ = (surface, view, command);
}

/// Draws the scroll indicator.
pub(crate) fn draw_indicator(cx: &mut DrawCx<'_>, state: &ViewState) {
    let _ = (cx, state);
}
