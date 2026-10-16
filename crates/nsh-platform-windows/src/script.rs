//! Scripted runs of the standalone app (`AppOptions::script`): steps
//! separated by `;` or new lines, run in order once the window shows, to
//! check the backend without touching the screen.
//!
//! - `wait <ms>`: waits.
//! - `shot <file.png>`: draws the window's content offscreen at scale 1
//!   into a PNG, whatever covers the window on screen.
//! - `tap <x> <y>`: presses and releases at a point (points, root
//!   coordinates).
//! - `drag <x0> <y0> <x1> <y1> [ms]`: presses, moves in steps over `ms`
//!   (default 300), releases.
//! - `touch <x0> <y0> <x1> <y1> [ms]`: the same with a finger.
//! - `wheel <x> <y> <dx> <dy>`: a wheel scroll (points; positive scrolls
//!   toward the end); `pad <x> <y> <dx> <dy>` a precision touchpad's.
//! - `hover <x> <y>`: moves the pointer.
//! - `type <text>`: types into the focused field.
//! - `key <name>`: presses a key: `Return`, `BackSpace`, `Tab`, `Escape`,
//!   `Left`, `Home`, `Page_Down`, `F5`, a letter, ...; `ctrl+`, `shift+`,
//!   `alt+` prefixes add modifiers (`ctrl+a`).
//! - `selectall`: Ctrl+A.
//! - `dark` / `light`: switches the system's appearance (an app that
//!   forces one keeps it).
//! - `resize <w> <h>`: resizes the window's client area (points).
//! - `quit`: closes the window.
//!
//! The steps run on a thread of their own; each one runs on the main
//! thread ([`crate::window::invoke`]) and the next waits for it.

use std::{rc::Rc, sync::mpsc, time::Duration};

use crate::{
    input::{KeyEvent, Modifiers, PointerEvent, PointerKind, WheelEvent},
    surface::Surface,
};

/// Runs `script`'s steps on `surface`.
pub(crate) fn run(surface: Rc<Surface>, script: &str) {
    let key = surface.key();
    let steps: Vec<String> = script
        .split([';', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    std::thread::spawn(move || {
        // The first frame needs the first commit.
        std::thread::sleep(Duration::from_millis(50));
        for step in steps {
            let mut words = step.splitn(2, ' ');
            let cmd = words.next().unwrap_or_default().to_owned();
            let rest = words.next().unwrap_or_default().trim().to_owned();
            let nums: Vec<f32> = rest
                .split_whitespace()
                .filter_map(|v| v.parse().ok())
                .collect();
            let num = |k: usize| nums.get(k).copied().unwrap_or(0.0);
            match cmd.as_str() {
                "wait" => std::thread::sleep(Duration::from_millis(num(0) as u64)),
                "tap" => {
                    let (x, y) = (num(0), num(1));
                    on_main(key, move |s| pointer(s, PointerKind::Down, x, y, false));
                    std::thread::sleep(Duration::from_millis(60));
                    on_main(key, move |s| pointer(s, PointerKind::Up, x, y, false));
                }
                "drag" | "touch" => {
                    let touch = cmd == "touch";
                    let (x0, y0, x1, y1) = (num(0), num(1), num(2), num(3));
                    let ms = if nums.len() > 4 { num(4) } else { 300.0 };
                    let steps = (ms / 16.0).ceil().max(1.0) as u32;
                    on_main(key, move |s| pointer(s, PointerKind::Down, x0, y0, touch));
                    for i in 1..=steps {
                        std::thread::sleep(Duration::from_millis(16));
                        let t = i as f32 / steps as f32;
                        let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
                        on_main(key, move |s| pointer(s, PointerKind::Move, x, y, touch));
                    }
                    on_main(key, move |s| pointer(s, PointerKind::Up, x1, y1, touch));
                }
                "hover" => {
                    let (x, y) = (num(0), num(1));
                    on_main(key, move |s| pointer(s, PointerKind::Move, x, y, false));
                }
                "wheel" | "pad" => {
                    let precise = cmd == "pad";
                    let (x, y, dx, dy) = (num(0), num(1), num(2), num(3));
                    on_main(key, move |s| {
                        crate::input::wheel(
                            s,
                            WheelEvent {
                                x,
                                y,
                                dx,
                                dy,
                                precise,
                                modifiers: Modifiers::default(),
                                time: s.clock(),
                            },
                        )
                    });
                }
                "type" => on_main(key, move |s| crate::text_input::char_input(s, &rest)),
                "key" | "selectall" => {
                    let name = if cmd == "selectall" {
                        "ctrl+a".to_owned()
                    } else {
                        rest.clone()
                    };
                    match parse_key(&name) {
                        Some((vk, modifiers)) => on_main(key, move |s| press(s, vk, modifiers)),
                        None => log::warn!("script: unknown key `{name}`"),
                    }
                }
                "dark" | "light" => {
                    let dark = cmd == "dark";
                    on_main(key, move |s| {
                        crate::system::simulate_system_scheme(dark);
                        crate::system::settings_changed(s);
                    });
                }
                "resize" => {
                    let (w, h) = (num(0), num(1));
                    on_main(key, move |s| resize(s, w, h));
                }
                "shot" => on_main(key, move |s| match shot(s, &rest) {
                    Ok(()) => log::info!("script: wrote {rest}"),
                    Err(err) => log::error!("script: shot {rest}: {err}"),
                }),
                "quit" => on_main(key, |s| {
                    // SAFETY: a plain call on the surface's window.
                    let _ = unsafe {
                        windows::Win32::UI::WindowsAndMessaging::DestroyWindow(s.hwnd())
                    };
                }),
                other => log::warn!("script: unknown step `{other}`"),
            }
        }
    });
}

/// Runs `f` with the surface on the main thread and waits for it.
fn on_main(key: u64, f: impl FnOnce(&Rc<Surface>) + Send + 'static) {
    let (tx, rx) = mpsc::channel();
    crate::window::invoke(move || {
        if let Some(surface) = Surface::by_key(key) {
            f(&surface);
        }
        let _ = tx.send(());
    });
    let _ = rx.recv();
}

fn pointer(surface: &Rc<Surface>, kind: PointerKind, x: f32, y: f32, touch: bool) {
    crate::input::pointer(
        surface,
        PointerEvent {
            kind,
            x,
            y,
            button: 0,
            pointer: if touch { 100 } else { 1 },
            touch,
            pen: false,
            time: surface.clock(),
            modifiers: Modifiers::default(),
            clicks: 1,
        },
    );
}

fn press(surface: &Rc<Surface>, vk: u16, modifiers: Modifiers) {
    for down in [true, false] {
        let handled = crate::input::key(
            surface,
            KeyEvent {
                vk,
                down,
                repeat: false,
                modifiers,
            },
        );
        // Unhandled printable keys type their character, as
        // `TranslateMessage` would.
        if down && !handled && !modifiers.ctrl && !modifiers.alt {
            let ch = match vk {
                0x20 => Some(' '),
                0x0D => Some('\r'),
                0x09 => Some('\t'),
                0x30..=0x39 => char::from_u32(vk as u32),
                0x41..=0x5A => char::from_u32(if modifiers.shift { vk } else { vk + 32 } as u32),
                _ => None,
            };
            if let Some(ch) = ch {
                crate::text_input::char_input(surface, &ch.to_string());
            }
        }
    }
}

/// A key name (with `ctrl+` / `shift+` / `alt+` prefixes) as a virtual
/// key and modifiers.
fn parse_key(name: &str) -> Option<(u16, Modifiers)> {
    let mut modifiers = Modifiers::default();
    let mut key = name;
    loop {
        let lower = key.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("ctrl+") {
            modifiers.ctrl = true;
            key = &key[key.len() - rest.len()..];
        } else if let Some(rest) = lower.strip_prefix("shift+") {
            modifiers.shift = true;
            key = &key[key.len() - rest.len()..];
        } else if let Some(rest) = lower.strip_prefix("alt+") {
            modifiers.alt = true;
            key = &key[key.len() - rest.len()..];
        } else {
            break;
        }
    }
    let vk = match key {
        "Return" | "Enter" => 0x0D,
        "BackSpace" => 0x08,
        "Tab" => 0x09,
        "Escape" => 0x1B,
        "space" | "Space" => 0x20,
        "Page_Up" | "PageUp" => 0x21,
        "Page_Down" | "PageDown" => 0x22,
        "End" => 0x23,
        "Home" => 0x24,
        "Left" => 0x25,
        "Up" => 0x26,
        "Right" => 0x27,
        "Down" => 0x28,
        "Delete" => 0x2E,
        "BrowserBack" => 0xA6,
        k if k.len() == 1 && k.as_bytes()[0].is_ascii_alphanumeric() => {
            k.as_bytes()[0].to_ascii_uppercase() as u16
        }
        k if k.starts_with('F') && k[1..].parse::<u16>().is_ok_and(|n| (1..=24).contains(&n)) => {
            0x70 + k[1..].parse::<u16>().ok()? - 1
        }
        _ => return None,
    };
    Some((vk, modifiers))
}

/// Resizes the window so that its client area is `w` x `h` points.
fn resize(surface: &Rc<Surface>, w: f32, h: f32) {
    use windows::Win32::{
        Foundation::RECT,
        UI::{
            HiDpi::AdjustWindowRectExForDpi,
            WindowsAndMessaging::{
                GWL_EXSTYLE, GWL_STYLE, GetWindowLongW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER,
                SetWindowPos, WINDOW_EX_STYLE, WINDOW_STYLE,
            },
        },
    };
    let hwnd = surface.hwnd();
    let dpi = crate::window::dpi(hwnd);
    let s = dpi as f32 / 96.0;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: (w * s).round() as i32,
        bottom: (h * s).round() as i32,
    };
    // SAFETY: plain calls on the surface's window.
    unsafe {
        let style = WINDOW_STYLE(GetWindowLongW(hwnd, GWL_STYLE) as u32);
        let ex = WINDOW_EX_STYLE(GetWindowLongW(hwnd, GWL_EXSTYLE) as u32);
        let _ = AdjustWindowRectExForDpi(&mut rect, style, false, ex, dpi);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            rect.right - rect.left,
            rect.bottom - rect.top,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

/// Draws the surface offscreen at scale 1 and writes it to `path` (PNG).
fn shot(surface: &Rc<Surface>, path: &str) -> Result<(), String> {
    let pixels = crate::render::snapshot(surface, 1.0).map_err(|e| e.to_string())?;
    let mut rgba = pixels.data;
    for px in rgba.chunks_exact_mut(4) {
        // Premultiplied BGRA to straight RGBA.
        let a = px[3];
        let un = |c: u8| {
            if a == 0 {
                0
            } else {
                ((c as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8
            }
        };
        let (b, g, r) = (un(px[0]), un(px[1]), un(px[2]));
        px[0] = r;
        px[1] = g;
        px[2] = b;
    }
    image::save_buffer(
        path,
        &rgba,
        pixels.width,
        pixels.height,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_key;

    #[test]
    fn key_names_map_to_virtual_keys() {
        assert_eq!(parse_key("Return").unwrap().0, 0x0D);
        assert_eq!(parse_key("F5").unwrap().0, 0x74);
        let (vk, m) = parse_key("ctrl+shift+z").unwrap();
        assert_eq!(vk, b'Z' as u16);
        assert!(m.ctrl && m.shift && !m.alt);
        assert!(parse_key("Nope").is_none());
    }
}
