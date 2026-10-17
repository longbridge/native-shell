//! GPUI's input events into a surface's input handlers ([`crate::input`]).
//!
//! [`install`] runs in the surface element's paint (GPUI's mouse and key
//! listeners last one frame): mouse presses on the surface's hitbox, and
//! moves and releases while a press that began there lasts (GPUI captures
//! nothing: the listeners see every event of the window), wheel and
//! touchpad scrolls the hitbox should handle, and keys while the surface's
//! focus handle is in the focus path. Positions become root coordinates
//! (the surface's origin in the window subtracted); times are seconds of
//! the surface's clock. Each handler ends with [`crate::rt::flush`].

use std::rc::Rc;

use gpui::{
    App, Bounds, CursorStyle, DispatchPhase, FocusHandle, Hitbox, KeyDownEvent, KeyUpEvent,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, ScrollDelta,
    ScrollWheelEvent, Window, px,
};

use crate::{
    input::{Cursor, KeyEvent, PointerEvent, PointerKind, WheelEvent},
    surface::Surface,
};

/// How many points a wheel line scrolls.
const LINE: f32 = 40.0;

fn button(b: MouseButton) -> Option<u8> {
    Some(match b {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Navigate(gpui::NavigationDirection::Back) => 3,
        MouseButton::Navigate(gpui::NavigationDirection::Forward) => 4,
    })
}

fn root_point(surface: &Surface, position: gpui::Point<Pixels>) -> (f32, f32) {
    let (ox, oy) = surface.origin.get();
    (f32::from(position.x) - ox, f32::from(position.y) - oy)
}

fn cursor_style(c: Cursor) -> CursorStyle {
    match c {
        Cursor::Arrow => CursorStyle::Arrow,
        Cursor::Hand => CursorStyle::PointingHand,
        Cursor::Text => CursorStyle::IBeam,
        Cursor::Crosshair => CursorStyle::Crosshair,
        Cursor::Move => CursorStyle::OpenHand,
        Cursor::NotAllowed => CursorStyle::OperationNotAllowed,
        Cursor::ResizeEw => CursorStyle::ResizeLeftRight,
        Cursor::ResizeNs => CursorStyle::ResizeUpDown,
        Cursor::Wait | Cursor::Progress | Cursor::Help => CursorStyle::Arrow,
        Cursor::Hidden => CursorStyle::Arrow,
    }
}

/// Listens to the window's input for `surface` this frame.
pub(crate) fn install(
    surface: &Rc<Surface>,
    focus: &FocusHandle,
    _view: &gpui::WeakEntity<crate::app::AppView>,
    hitbox: &Hitbox,
    _bounds: Bounds<Pixels>,
    window: &mut Window,
    _cx: &mut App,
) {
    if hitbox.is_hovered(window) {
        let (x, y) = root_point(surface, window.mouse_position());
        let c = crate::input::cursor(surface, x, y);
        window.set_cursor_style(cursor_style(c), hitbox);
    }

    let s = surface.clone();
    let h = hitbox.clone();
    let f = focus.clone();
    window.on_mouse_event(move |e: &MouseDownEvent, phase, window, cx| {
        if phase != DispatchPhase::Bubble || !h.is_hovered(window) {
            return;
        }
        let Some(button) = button(e.button) else {
            return;
        };
        if !f.is_focused(window) {
            window.focus(&f, cx);
        }
        let (x, y) = root_point(&s, e.position);
        if button == 0 && crate::input::window_drag(&s, x, y) {
            window.start_window_move();
            return;
        }
        s.input_pressed.set(s.input_pressed.get() | 1 << button);
        crate::input::pointer(
            &s,
            PointerEvent {
                kind: PointerKind::Down,
                x,
                y,
                button,
                pointer: 1,
                touch: false,
                pen: false,
                time: s.clock(),
                modifiers: crate::keys::modifiers(&e.modifiers),
                clicks: e.click_count as u32,
            },
        );
        cx.stop_propagation();
        crate::rt::flush(cx);
    });

    let s = surface.clone();
    window.on_mouse_event(move |e: &MouseUpEvent, phase, _window, cx| {
        if phase != DispatchPhase::Bubble {
            return;
        }
        let Some(button) = button(e.button) else {
            return;
        };
        let pressed = s.input_pressed.get();
        if pressed & 1 << button == 0 {
            return;
        }
        s.input_pressed.set(pressed & !(1 << button));
        let (x, y) = root_point(&s, e.position);
        crate::input::pointer(
            &s,
            PointerEvent {
                kind: PointerKind::Up,
                x,
                y,
                button,
                pointer: 1,
                touch: false,
                pen: false,
                time: s.clock(),
                modifiers: crate::keys::modifiers(&e.modifiers),
                clicks: e.click_count as u32,
            },
        );
        crate::rt::flush(cx);
    });

    let s = surface.clone();
    let h = hitbox.clone();
    window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, cx| {
        if phase != DispatchPhase::Bubble {
            return;
        }
        let hovered = h.is_hovered(window);
        let pressed = s.input_pressed.get() != 0;
        let (x, y) = root_point(&s, e.position);
        let kind = if hovered || pressed {
            s.input_hovered.set(true);
            PointerKind::Move
        } else if s.input_hovered.replace(false) {
            PointerKind::Leave
        } else {
            return;
        };
        crate::input::pointer(
            &s,
            PointerEvent {
                kind,
                x,
                y,
                button: 0,
                pointer: 1,
                touch: false,
                pen: false,
                time: s.clock(),
                modifiers: crate::keys::modifiers(&e.modifiers),
                clicks: 0,
            },
        );
        crate::rt::flush(cx);
    });

    let s = surface.clone();
    let h = hitbox.clone();
    window.on_mouse_event(move |e: &ScrollWheelEvent, phase, window, cx| {
        if phase != DispatchPhase::Bubble || !h.should_handle_scroll(window) {
            return;
        }
        let (x, y) = root_point(&s, e.position);
        // GPUI's deltas move the content (positive: down / right); nsh's
        // scroll toward the end.
        let (dx, dy, precise) = match e.delta {
            ScrollDelta::Pixels(p) => (-f32::from(p.x), -f32::from(p.y), true),
            ScrollDelta::Lines(l) => (-l.x * LINE, -l.y * LINE, false),
        };
        crate::input::wheel(
            &s,
            WheelEvent {
                x,
                y,
                dx,
                dy,
                precise,
                modifiers: crate::keys::modifiers(&e.modifiers),
                time: s.clock(),
            },
        );
        cx.stop_propagation();
        crate::rt::flush(cx);
    });

    let s = surface.clone();
    window.on_key_event(move |e: &KeyDownEvent, phase, window, cx| {
        if phase != DispatchPhase::Bubble {
            return;
        }
        let Some(vk) = crate::keys::vk_of(&e.keystroke.key) else {
            return;
        };
        let handled = crate::input::key(
            &s,
            KeyEvent {
                vk,
                down: true,
                repeat: e.is_held,
                modifiers: crate::keys::modifiers(&e.keystroke.modifiers),
            },
        );
        if handled {
            cx.stop_propagation();
            window.prevent_default();
        }
        crate::rt::flush(cx);
    });

    let s = surface.clone();
    window.on_key_event(move |e: &KeyUpEvent, phase, _window, cx| {
        if phase != DispatchPhase::Bubble {
            return;
        }
        let Some(vk) = crate::keys::vk_of(&e.keystroke.key) else {
            return;
        };
        crate::input::key(
            &s,
            KeyEvent {
                vk,
                down: false,
                repeat: false,
                modifiers: crate::keys::modifiers(&e.keystroke.modifiers),
            },
        );
        crate::rt::flush(cx);
    });
    let _ = px(0.0);
}
