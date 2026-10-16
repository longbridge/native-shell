//! The environment (appearance, locale, scale, accessibility settings),
//! dialogs, toasts, the clipboard and the system's other services.

use std::rc::Rc;

use nsh_core::{Command, NodeId};
use nsh_engine::ColorScheme;
use objc2_app_kit::NSAppearanceCustomization;

use crate::surface::Surface;

/// The window or its screen changed (scale, ...).
pub(crate) fn window_changed(_surface: &Rc<Surface>) {}

/// The system's appearance changed.
pub(crate) fn appearance_changed(surface: &Rc<Surface>) {
    environment_changed(surface);
}

/// Reads the environment again and tells the engine.
fn environment_changed(surface: &Rc<Surface>) {
    let mut env = surface.environment.get();
    env.color_scheme = color_scheme(surface);
    surface.set_environment(env);
}

/// The color scheme: the app's appearance, else the system's.
pub(crate) fn color_scheme(surface: &Surface) -> ColorScheme {
    use nsh_core::platform::Appearance;
    match surface.appearance.get() {
        Appearance::Light => ColorScheme::Light,
        Appearance::Dark => ColorScheme::Dark,
        Appearance::System => {
            let dark = surface
                .root
                .ns()
                .effectiveAppearance()
                .name()
                .to_string()
                .contains("Dark");
            if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            }
        }
    }
}

/// Starts following the system's settings.
pub(crate) fn watch_environment(surface: &Rc<Surface>) {
    environment_changed(surface);
}

/// The engine was told about a new environment.
pub(crate) fn environment_applied(_surface: &Surface) {}

/// Dialogs, toasts, the clipboard, ...
pub(crate) fn command(_surface: &Rc<Surface>, _id: NodeId, command: &Command) {
    log::debug!("unhandled command {command:?}");
}

/// A batch was applied.
pub(crate) fn end_batch(_surface: &Surface) {}

/// The surface got a new size.
pub(crate) fn surface_resized(_surface: &Surface, _width: f32, _height: f32) {}

/// The app stopped: dialogs and toasts go.
pub(crate) fn clear(_surface: &Surface) {}

/// The app's appearance (`setColorScheme`, `nsh.json`).
pub(crate) fn set_appearance(surface: &Rc<Surface>, appearance: nsh_core::platform::Appearance) {
    surface.appearance.set(appearance);
    environment_changed(surface);
}

/// Gives a new window the app's appearance (`nsh.json`), so it opens in
/// it rather than flashing the system's.
pub(crate) fn open_window(
    window: &objc2_app_kit::NSWindow,
    appearance: nsh_core::platform::Appearance,
) {
    use nsh_core::platform::Appearance;
    use objc2_app_kit::{NSAppearance, NSAppearanceNameAqua, NSAppearanceNameDarkAqua};
    // SAFETY: immutable AppKit constants.
    let name = match appearance {
        Appearance::Light => unsafe { NSAppearanceNameAqua },
        Appearance::Dark => unsafe { NSAppearanceNameDarkAqua },
        Appearance::System => return,
    };
    window.setAppearance(NSAppearance::appearanceNamed(name).as_deref());
}
