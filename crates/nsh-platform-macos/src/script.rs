//! Steps run once the window shows (`--script`, [`crate::AppOptions::script`]):
//! screenshots and simulated input, to check a change without touching
//! the screen.
//!
//! Steps are separated by `;` or new lines:
//!
//! | Step | Does |
//! |---|---|
//! | `wait <ms>` | Waits. |
//! | `shot <file.png>` | Renders the window's content to a PNG (in process, whatever covers the window). |
//! | `tap <x> <y>` | Presses and releases the left button at a point (points, from the content's top left). |
//! | `drag <x0> <y0> <x1> <y1> [ms]` | Presses, moves in steps over `ms` (default 300), releases. |
//! | `wheel <x> <y> <dx> <dy>` | A precise (trackpad) scroll by `(dx, dy)` points: positive moves the content up / left. |
//! | `hover <x> <y>` | Moves the pointer. |
//! | `type <text>` | Types into the focused field. |
//! | `key <name>` | Presses a key: `Return`, `Escape`, `Tab`, `BackSpace`, `Up`, `Down`, `Left`, `Right`, `Page_Up`, `Page_Down`, `Home`, `End`, `space`, or one character. |
//! | `dark` / `light` | Switches the app's window to that appearance (as the system's would). |
//! | `resize <w> <h>` | Resizes the window's content. |
//! | `quit` | Closes the window. |
//!
//! Events go through the window (`sendEvent:`), so they take the path
//! real ones take: AppKit's hit testing, the root view, [`crate::input`].

use std::{rc::Rc, time::Duration};

use dispatch2::{DispatchQueue, DispatchTime};
use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSEvent, NSEventModifierFlags, NSEventType, NSWindow,
};
use objc2_core_foundation::CGFloat;
use objc2_foundation::{NSPoint, NSSize, NSString};

use crate::surface::Surface;

/// One step of a script.
#[derive(Debug, Clone, PartialEq)]
enum Step {
    Wait(u64),
    Shot(String),
    Tap(f64, f64),
    Drag(f64, f64, f64, f64, u64),
    Wheel(f64, f64, f64, f64),
    Hover(f64, f64),
    /// A drag's move (internal: [`Step::Drag`] queues them).
    DragTo(f64, f64),
    /// A drag's release (internal).
    Release(f64, f64),
    Type(String),
    Key(String),
    Appearance(bool),
    Resize(f64, f64),
    Quit,
}

/// Parses `script` (unknown steps are logged and skipped).
fn parse(script: &str) -> Vec<Step> {
    let mut steps = Vec::new();
    for raw in script.split([';', '\n']) {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let (name, rest) = raw.split_once(' ').unwrap_or((raw, ""));
        let rest = rest.trim();
        let nums: Vec<f64> = rest
            .split_whitespace()
            .filter_map(|n| n.parse().ok())
            .collect();
        let step = match (name, nums.as_slice()) {
            ("wait", [ms]) => Some(Step::Wait(*ms as u64)),
            ("shot", _) if !rest.is_empty() => Some(Step::Shot(rest.to_owned())),
            ("tap", [x, y]) => Some(Step::Tap(*x, *y)),
            ("drag", [x0, y0, x1, y1]) => Some(Step::Drag(*x0, *y0, *x1, *y1, 300)),
            ("drag", [x0, y0, x1, y1, ms]) => Some(Step::Drag(*x0, *y0, *x1, *y1, *ms as u64)),
            ("wheel", [x, y, dx, dy]) => Some(Step::Wheel(*x, *y, *dx, *dy)),
            ("hover", [x, y]) => Some(Step::Hover(*x, *y)),
            ("type", _) => Some(Step::Type(rest.to_owned())),
            ("key", _) if !rest.is_empty() => Some(Step::Key(rest.to_owned())),
            ("dark", []) => Some(Step::Appearance(true)),
            ("light", []) => Some(Step::Appearance(false)),
            ("resize", [w, h]) => Some(Step::Resize(*w, *h)),
            ("quit", []) => Some(Step::Quit),
            _ => None,
        };
        match step {
            Some(step) => steps.push(step),
            None => log::warn!("script: cannot read step `{raw}`"),
        }
    }
    steps
}

/// Runs `script`'s steps in `window`, one after the other, starting after
/// the window's first frames.
pub(crate) fn run(window: &NSWindow, surface: Rc<Surface>, script: &str) {
    let steps = parse(script);
    let window = Retained::from(window);
    let key = surface.key();
    drop(surface);
    // The window number identifies it for `screencapture`; the rest needs
    // the window itself, on the main thread.
    let state = Rc::new(Runner {
        window,
        _surface: key,
        steps: std::cell::RefCell::new(steps.into_iter().collect()),
    });
    RUNNER.with(|r| *r.borrow_mut() = Some(state));
    later(300, next);
}

struct Runner {
    window: Retained<NSWindow>,
    /// The surface's key (kept for steps that will need it).
    _surface: u64,
    steps: std::cell::RefCell<std::collections::VecDeque<Step>>,
}

thread_local! {
    static RUNNER: std::cell::RefCell<Option<Rc<Runner>>> = const { std::cell::RefCell::new(None) };
}

/// Calls `f` on the main thread after `ms` milliseconds.
fn later(ms: u64, f: fn()) {
    let when = DispatchTime::NOW.time(Duration::from_millis(ms).as_nanos() as i64);
    if DispatchQueue::main().after(when, f).is_err() {
        DispatchQueue::main().exec_async(f);
    }
}

/// Runs the next step, then schedules the one after it.
fn next() {
    let Some(runner) = RUNNER.with(|r| r.borrow().clone()) else {
        return;
    };
    let Some(step) = runner.steps.borrow_mut().pop_front() else {
        return;
    };
    let delay = run_step(&runner, step);
    if let Some(delay) = delay {
        later(delay, next);
    }
}

/// Runs `step`; returns the delay before the next one (`None`: no next).
fn run_step(runner: &Runner, step: Step) -> Option<u64> {
    let window = &*runner.window;
    match step {
        Step::Wait(ms) => return Some(ms),
        Step::Shot(path) => shot(window, &path),
        Step::Tap(x, y) => {
            mouse(window, NSEventType::LeftMouseDown, x, y, 1);
            mouse(window, NSEventType::LeftMouseUp, x, y, 1);
        }
        Step::Drag(x0, y0, x1, y1, ms) => {
            mouse(window, NSEventType::LeftMouseDown, x0, y0, 1);
            // The moves run as steps of their own, a frame apart.
            let frames = (ms / 16).max(1);
            let mut queued: Vec<Step> = Vec::new();
            for i in 1..=frames {
                let t = i as f64 / frames as f64;
                queued.push(Step::DragTo(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t));
            }
            queued.push(Step::Release(x1, y1));
            let mut steps = runner.steps.borrow_mut();
            for s in queued.into_iter().rev() {
                steps.push_front(s);
            }
        }
        Step::DragTo(x, y) => mouse(window, NSEventType::LeftMouseDragged, x, y, 1),
        Step::Release(x, y) => mouse(window, NSEventType::LeftMouseUp, x, y, 1),
        Step::Hover(x, y) => mouse(window, NSEventType::MouseMoved, x, y, 0),
        Step::Wheel(x, y, dx, dy) => wheel(window, x, y, dx, dy),
        Step::Type(text) => type_text(window, &text),
        Step::Key(name) => key(window, &name),
        Step::Appearance(dark) => {
            // SAFETY: immutable AppKit constants.
            let name = if dark {
                unsafe { NSAppearanceNameDarkAqua }
            } else {
                unsafe { NSAppearanceNameAqua }
            };
            window.setAppearance(NSAppearance::appearanceNamed(name).as_deref());
        }
        Step::Resize(w, h) => window.setContentSize(NSSize::new(w, h)),
        Step::Quit => {
            window.close();
            return None;
        }
    }
    Some(16)
}

/// A point of the content view (points from its top left) in window
/// coordinates (from its bottom left).
fn to_window(window: &NSWindow, x: f64, y: f64) -> NSPoint {
    let height = window.contentView().map_or(0.0, |v| v.frame().size.height);
    NSPoint::new(x as CGFloat, height - y as CGFloat)
}

fn mouse(window: &NSWindow, kind: NSEventType, x: f64, y: f64, clicks: isize) {
    let event = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
        kind,
        to_window(window, x, y),
        NSEventModifierFlags::empty(),
        timestamp(),
        window.windowNumber(),
        None,
        0,
        clicks,
        if clicks > 0 { 1.0 } else { 0.0 },
    );
    if let Some(event) = event {
        window.sendEvent(&event);
    }
}

/// Seconds since the system started, as event timestamps are.
fn timestamp() -> f64 {
    objc2_foundation::NSProcessInfo::processInfo().systemUptime()
}

fn wheel(window: &NSWindow, x: f64, y: f64, dx: f64, dy: f64) {
    use objc2_core_graphics::{CGEvent, CGEventField, CGScrollEventUnit};
    // A pixel (precise) scroll, as a trackpad sends: content follows the
    // fingers, so a positive `dy` (content up) is a negative delta.
    let Some(event) = CGEvent::new_scroll_wheel_event2(
        None,
        CGScrollEventUnit::Pixel,
        2,
        -dy.round() as i32,
        -dx.round() as i32,
        0,
    ) else {
        return;
    };
    let p = to_window(window, x, y);
    let screen = window.convertPointToScreen(p);
    let main_height = objc2_app_kit::NSScreen::screens(MainThreadMarker::from(window))
        .firstObject()
        .map_or(0.0, |s| s.frame().size.height);
    CGEvent::set_location(
        Some(&event),
        objc2_core_foundation::CGPoint::new(screen.x, main_height - screen.y),
    );
    // Phase "changed" (a trackpad gesture, not a notched wheel).
    CGEvent::set_integer_value_field(Some(&event), CGEventField::ScrollWheelEventScrollPhase, 2);
    if let Some(ns) = NSEvent::eventWithCGEvent(&event) {
        window.sendEvent(&ns);
    }
}

fn type_text(window: &NSWindow, text: &str) {
    for c in text.chars() {
        let s = c.to_string();
        key_event(window, &s, 0);
    }
}

fn key(window: &NSWindow, name: &str) {
    // (characters, virtual key code)
    let (chars, code): (String, u16) = match name {
        "Return" => ("\r".into(), 36),
        "Escape" => ("\u{1b}".into(), 53),
        "Tab" => ("\t".into(), 48),
        "BackSpace" => ("\u{7f}".into(), 51),
        "space" => (" ".into(), 49),
        "Up" => ("\u{f700}".into(), 126),
        "Down" => ("\u{f701}".into(), 125),
        "Left" => ("\u{f702}".into(), 123),
        "Right" => ("\u{f703}".into(), 124),
        "Page_Up" => ("\u{f72c}".into(), 116),
        "Page_Down" => ("\u{f72d}".into(), 121),
        "Home" => ("\u{f729}".into(), 115),
        "End" => ("\u{f72b}".into(), 119),
        other => (other.into(), 0),
    };
    key_event(window, &chars, code);
}

fn key_event(window: &NSWindow, chars: &str, code: u16) {
    let s = NSString::from_str(chars);
    for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
        let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
            kind,
            NSPoint::new(0.0, 0.0),
            NSEventModifierFlags::empty(),
            timestamp(),
            window.windowNumber(),
            None,
            &s,
            &s,
            false,
            code,
        );
        if let Some(event) = event {
            window.sendEvent(&event);
        }
    }
}

/// Renders the window's content into a PNG (in process: no Screen
/// Recording permission, and it works while the screen is locked).
fn shot(window: &NSWindow, path: &str) {
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRepPropertyKey};
    use objc2_foundation::NSDictionary;
    let Some(view) = window.contentView() else {
        return;
    };
    let bounds = view.bounds();
    let Some(rep) = view.bitmapImageRepForCachingDisplayInRect(bounds) else {
        return log::error!("script: cannot make a bitmap for {path}");
    };
    view.cacheDisplayInRect_toBitmapImageRep(bounds, &rep);
    let properties = NSDictionary::<NSBitmapImageRepPropertyKey, objc2::runtime::AnyObject>::new();
    // SAFETY: an empty property dictionary is valid for PNG.
    let data =
        unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &properties) };
    match data.map(|d| std::fs::write(path, d.to_vec())) {
        Some(Ok(())) => log::info!("script: saved {path}"),
        other => log::error!("script: cannot save {path}: {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_steps() {
        assert_eq!(
            parse("wait 500; shot a.png\ntap 10 20; drag 1 2 3 4 100; key Return; quit; nope"),
            vec![
                Step::Wait(500),
                Step::Shot("a.png".into()),
                Step::Tap(10.0, 20.0),
                Step::Drag(1.0, 2.0, 3.0, 4.0, 100),
                Step::Key("Return".into()),
                Step::Quit,
            ]
        );
    }
}
