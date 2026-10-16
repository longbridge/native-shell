//! Pointer, wheel and keyboard input: hit testing over the views, touches
//! for the responder system, native presses, hover and pressed states,
//! gestures and scrolling.
//!
//! The window procedure ([`crate::window`]) turns Win32 messages into the
//! events here (root coordinates, points).

use std::rc::Rc;

use nsh_core::{Command, GestureValues, NodeId};

use crate::{surface::Surface, view::NshView};

/// What a pointer did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointerKind {
    /// A button (or a finger, a pen) went down.
    Down,
    /// It moved (with or without buttons).
    Move,
    /// The button went up.
    Up,
    /// The gesture was taken away (capture lost).
    Cancel,
    /// The mouse left the window.
    Leave,
}

/// Modifier keys.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Modifiers {
    /// Shift.
    pub(crate) shift: bool,
    /// Ctrl.
    pub(crate) ctrl: bool,
    /// Alt.
    pub(crate) alt: bool,
    /// The Windows key.
    pub(crate) meta: bool,
}

/// A mouse, touch or pen event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PointerEvent {
    /// What happened.
    pub(crate) kind: PointerKind,
    /// Where (points, root coordinates).
    pub(crate) x: f32,
    /// Where.
    pub(crate) y: f32,
    /// The button: 0 left, 1 right, 2 middle, 3 back (X1), 4 forward (X2).
    pub(crate) button: u8,
    /// Which pointer: 1 is the mouse, touch and pen contacts have their own.
    pub(crate) pointer: u32,
    /// A finger.
    pub(crate) touch: bool,
    /// A pen.
    pub(crate) pen: bool,
    /// When (seconds, the surface's clock).
    pub(crate) time: f64,
    /// Modifiers held.
    pub(crate) modifiers: Modifiers,
    /// Consecutive clicks at the same place (presses).
    pub(crate) clicks: u32,
}

/// A wheel or touchpad scroll.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WheelEvent {
    /// Where the pointer is (points, root coordinates).
    pub(crate) x: f32,
    /// Where.
    pub(crate) y: f32,
    /// How far to scroll the content (points; positive toward the end:
    /// right, down).
    pub(crate) dx: f32,
    /// How far, vertically.
    pub(crate) dy: f32,
    /// From a precision touchpad or a high-resolution wheel (not whole
    /// notches).
    pub(crate) precise: bool,
    /// Modifiers held (Ctrl + wheel zooms, Shift + wheel scrolls
    /// sideways).
    pub(crate) modifiers: Modifiers,
    /// When (seconds, the surface's clock).
    pub(crate) time: f64,
}

/// A key going down or up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct KeyEvent {
    /// The virtual key code (`VK_*`).
    pub(crate) vk: u16,
    /// Down (or up).
    pub(crate) down: bool,
    /// An auto-repeat of a key held down.
    pub(crate) repeat: bool,
    /// Modifiers held.
    pub(crate) modifiers: Modifiers,
}

/// The mouse cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Cursor {
    /// The arrow.
    #[default]
    Arrow,
    /// A hand (links, pressables with `cursor: pointer`).
    Hand,
    /// The I-beam (text).
    Text,
    /// A crosshair.
    Crosshair,
    /// Four arrows.
    Move,
    /// Not allowed.
    NotAllowed,
    /// Resize left and right.
    ResizeEw,
    /// Resize up and down.
    ResizeNs,
    /// Busy.
    Wait,
    /// Working in the background.
    Progress,
    /// Help.
    Help,
}

/// The pointer input state of a surface.
#[derive(Default)]
pub(crate) struct Input {}

/// A pointer event from the window.
pub(crate) fn pointer(surface: &Rc<Surface>, event: PointerEvent) {
    let _ = (surface, event);
}

/// A wheel event from the window.
pub(crate) fn wheel(surface: &Rc<Surface>, event: WheelEvent) {
    let _ = (surface, event);
}

/// A key from the window; true when handled.
pub(crate) fn key(surface: &Rc<Surface>, event: KeyEvent) -> bool {
    let _ = (surface, event);
    false
}

/// The cursor over `(x, y)` (root coordinates).
pub(crate) fn cursor(surface: &Rc<Surface>, x: f32, y: f32) -> Cursor {
    let _ = (surface, x, y);
    Cursor::Arrow
}

/// Forgets every gesture (the app stopped).
pub(crate) fn reset(surface: &Surface) {
    let _ = surface;
}

/// `TrackStates`.
pub(crate) fn track_states(surface: &Rc<Surface>, view: &NshView, states: u8) {
    let _ = surface;
    view.state_mut().tracked = states;
}

/// `Gestures`.
pub(crate) fn gestures(surface: &Rc<Surface>, view: &NshView, values: GestureValues) {
    let _ = (surface, view, values);
}

/// The view's events changed.
pub(crate) fn events_changed(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// The view's props changed.
pub(crate) fn props_changed(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// The view's style changed.
pub(crate) fn style_changed(view: &NshView) {
    let _ = view;
}

/// `SetJsResponder` / `ClearJsResponder`.
pub(crate) fn command(surface: &Rc<Surface>, id: NodeId, command: &Command) {
    let _ = (surface, id, command);
}
