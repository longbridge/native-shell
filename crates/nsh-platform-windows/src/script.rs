//! `--script` steps: screenshots and simulated input, to check the backend
//! without touching the screen.
//!
//! Steps are separated by `;` or new lines and run once the window shows,
//! one after another, on a thread of their own: each step runs on the main
//! thread ([`crate::window::invoke`]) and the next waits for it.
//!
//! | Step | Does |
//! |---|---|
//! | `wait <ms>` | Waits. |
//! | `shot <file.png>` | Draws the window's content offscreen at scale 1 into a PNG. |
//! | `quit` | Closes the window. |

use std::{rc::Rc, sync::mpsc, time::Duration};

use crate::surface::Surface;

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
        for step in steps {
            let (cmd, arg) = step.split_once(' ').unwrap_or((&step, ""));
            let arg = arg.trim().to_owned();
            match cmd {
                "wait" => {
                    std::thread::sleep(Duration::from_millis(arg.parse().unwrap_or(0)));
                    continue;
                }
                "shot" | "quit" => {}
                other => {
                    log::warn!("script: unknown step `{other}`");
                    continue;
                }
            }
            let cmd = cmd.to_owned();
            let (tx, rx) = mpsc::channel();
            crate::window::invoke(move || {
                if let Some(surface) = Surface::by_key(key) {
                    step_on_main(&surface, &cmd, &arg);
                }
                let _ = tx.send(());
            });
            let _ = rx.recv();
        }
    });
}

fn step_on_main(surface: &Rc<Surface>, cmd: &str, arg: &str) {
    match cmd {
        "shot" => match shot(surface, arg) {
            Ok(()) => log::info!("script: wrote {arg}"),
            Err(err) => log::error!("script: shot {arg}: {err}"),
        },
        "quit" => {
            // SAFETY: a plain call on the surface's window.
            let _ = unsafe {
                windows::Win32::UI::WindowsAndMessaging::DestroyWindow(surface.hwnd())
            };
        }
        _ => {}
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
