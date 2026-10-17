//! The environment, dialogs, toasts, the clipboard. (Being ported: see
//! `nsh-platform-windows/src/system.rs`.)

use std::rc::Rc;

use nsh_core::{Color, Command, NodeId, platform::Appearance};
use nsh_engine::ColorScheme;

use crate::{input::KeyEvent, surface::Surface};

/// True when the surface draws in its dark appearance.
pub(crate) fn dark(surface: &Surface) -> bool {
    surface.environment.get().color_scheme == ColorScheme::Dark
}

/// What the surface is drawn on.
pub(crate) fn window_background(surface: &Surface) -> Color {
    if dark(surface) {
        Color(0x0a0a_0aff)
    } else {
        Color(0xffff_ffff)
    }
}

/// The surface is about to be drawn in `window`: follows the window's
/// appearance.
pub(crate) fn window_prepaint(
    surface: &Rc<Surface>,
    window: &mut gpui::Window,
    _cx: &mut gpui::App,
) {
    let system_dark = matches!(
        window.appearance(),
        gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
    );
    let scheme = match surface.appearance.get() {
        Appearance::Dark => ColorScheme::Dark,
        Appearance::Light => ColorScheme::Light,
        _ if system_dark => ColorScheme::Dark,
        _ => ColorScheme::Light,
    };
    let mut env = surface.environment.get();
    if env.color_scheme != scheme {
        env.color_scheme = scheme;
        surface.set_environment(env);
    }
}

/// `Appearance`: the app's appearance.
pub(crate) fn set_appearance(surface: &Rc<Surface>, appearance: Appearance) {
    surface.appearance.set(appearance);
    surface.invalidate();
}

/// Root commands: dialogs, toasts, the clipboard.
pub(crate) fn command(surface: &Rc<Surface>, _id: NodeId, command: &Command) {
    match command {
        Command::Announce(text) => crate::access::announce(surface, text),
        _ => log::debug!("system: {command:?} is not supported yet"),
    }
}

/// The batch is applied.
pub(crate) fn end_batch(_surface: &Surface) {}

/// Keys dialogs take. True when taken.
pub(crate) fn key(_surface: &Rc<Surface>, _event: &KeyEvent) -> bool {
    false
}

/// The app stopped.
pub(crate) fn clear(_surface: &Surface) {}

/// The environment changed.
pub(crate) fn environment_applied(surface: &Surface) {
    surface.invalidate();
}

/// The surface was resized.
pub(crate) fn surface_resized(_surface: &Surface, _width: f32, _height: f32) {}
