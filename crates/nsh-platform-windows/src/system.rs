//! The environment (color scheme, scale, locale, text scale, reduce
//! motion, screen reader), dialogs, toasts and the clipboard.

use std::rc::Rc;

use nsh_core::{Color, Command, NodeId, platform::Appearance};
use nsh_engine::ColorScheme;
use windows::{
    Win32::{
        System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW},
        UI::Shell::SetCurrentProcessExplicitAppUserModelID,
    },
    core::{HSTRING, w},
};

use crate::surface::Surface;

/// True when the system's apps use the dark theme (Settings >
/// Personalization > Colors).
pub(crate) fn system_dark() -> bool {
    let mut value: u32 = 1;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: a DWORD out value of the size given.
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    result.is_ok() && value == 0
}

/// True when the surface draws in its dark appearance.
pub(crate) fn dark(surface: &Surface) -> bool {
    match surface.appearance.get() {
        Appearance::Dark => true,
        Appearance::Light => false,
        Appearance::System => system_dark(),
    }
}

/// The color behind the app's views.
pub(crate) fn window_background(surface: &Surface) -> Color {
    if dark(surface) {
        Color(0x0000_00ff)
    } else {
        Color(0xffff_ffff)
    }
}

/// Reads the environment and keeps it current.
pub(crate) fn watch_environment(surface: &Rc<Surface>) {
    settings_changed(surface);
}

/// A system setting changed (`WM_SETTINGCHANGE`, `WM_THEMECHANGED`).
pub(crate) fn settings_changed(surface: &Rc<Surface>) {
    let mut env = surface.environment.get();
    env.color_scheme = if dark(surface) {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    };
    surface.set_environment(env);
    environment_applied(surface);
    surface.invalidate();
}

/// The environment changed: the window follows it.
pub(crate) fn environment_applied(surface: &Surface) {
    let hwnd = surface.hwnd();
    if !hwnd.is_invalid() {
        crate::window::set_dark_title_bar(hwnd, dark(surface));
    }
}

/// `Appearance`: the app's appearance changed.
pub(crate) fn set_appearance(surface: &Rc<Surface>, appearance: Appearance) {
    surface.appearance.set(appearance);
    settings_changed(surface);
}

/// The taskbar groups the process's windows under `id`.
pub(crate) fn set_app_id(id: &str) {
    // SAFETY: a valid string.
    let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(id)) };
}

/// System commands (alerts, toasts, the clipboard, ...).
pub(crate) fn command(surface: &Rc<Surface>, id: NodeId, command: &Command) {
    let _ = (surface, id, command);
}

/// A batch was applied.
pub(crate) fn end_batch(surface: &Surface) {
    let _ = surface;
}

/// The surface was resized.
pub(crate) fn surface_resized(surface: &Surface, width: f32, height: f32) {
    let _ = (surface, width, height);
}

/// The app stopped: dialogs and toasts go.
pub(crate) fn clear(surface: &Surface) {
    let _ = surface;
}
