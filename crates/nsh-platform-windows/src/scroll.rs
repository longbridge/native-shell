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

/// True when `view` can scroll by `(dx, dy)` (not at its end that way).
pub(crate) fn can_scroll(view: &NshView, dx: f32, dy: f32) -> bool {
    let _ = (view, dx, dy);
    false
}

/// A drag (mouse, finger, pen) starts scrolling `view`.
pub(crate) fn drag_begin(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// The drag moved by `(dx, dy)` points since the last move (content
/// follows the pointer: positive scrolls toward the start).
pub(crate) fn drag_move(surface: &Rc<Surface>, view: &NshView, dx: f32, dy: f32) {
    let _ = (surface, view, dx, dy);
}

/// The drag ended with a velocity (points per second): a glide follows.
pub(crate) fn drag_end(surface: &Rc<Surface>, view: &NshView, vx: f32, vy: f32) {
    let _ = (surface, view, vx, vy);
}

/// A wheel or touchpad scroll starts on `view`.
pub(crate) fn wheel_begin(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// Scrolls `view` by `(dx, dy)` points (positive toward the end).
pub(crate) fn wheel(surface: &Rc<Surface>, view: &NshView, dx: f32, dy: f32) {
    let _ = (surface, view, dx, dy);
}

/// The wheel scroll ended (velocity in points per second).
pub(crate) fn wheel_end(surface: &Rc<Surface>, view: &NshView, vx: f32, vy: f32) {
    let _ = (surface, view, vx, vy);
}

/// Keyboard scrolling (arrows, Page Up / Down, Space, Home / End) of the
/// scroll view under `(x, y)` (root coordinates) or around the focus;
/// true when handled.
pub(crate) fn key(surface: &Rc<Surface>, event: &crate::input::KeyEvent, x: f32, y: f32) -> bool {
    let _ = (surface, event, x, y);
    false
}
